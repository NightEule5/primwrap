// SPDX-License-Identifier: Apache-2.0

use std::borrow::Cow;
use FnSelfArg::MutSelf;
use virtue::generate::{FnBuilder, ImplFor};
use virtue::prelude::*;
use virtue::prelude::FnSelfArg::{MutTakeSelf, RefSelf};
use crate::ops::{Accumulating, Binary, Comparing, OpData};

pub struct Sink<'a> {
	gen: Generator,
	op: Option<&'a dyn OpData>,
}

pub struct Impl<'a> {
	fn_name: &'a str,
	builder: ImplFor<'a, Generator>
}

pub struct Fn<'a: 'b, 'b>(FnBuilder<'b, ImplFor<'a, Generator>>);

impl From<Generator> for Sink<'_> {
	fn from(gen: Generator) -> Self {
		Self {
			gen,
			op: None,
		}
	}
}

impl<'a> Sink<'a> {
	pub fn binary(
		&mut self,
		data: &'a dyn OpData,
		ops: &[Binary]
	) -> &mut Self {
		let assign_data = data.assign();
		let assign_fn = format!(
			"{}::{}",
			assign_data.trait_name(),
			assign_data.fn_name()
		);

		self.push(data);
		for Binary { target, output, lhs, rhs_bind, rhs_type, base_ret, .. } in ops {
			let ret: Cow<_> = if let Some(ret) = base_ret {
				ret.to_string().into()
			} else {
				"self".into()
			};

			self.push_impl(target, Some(rhs_type))
				.output_type(output)
				.push_op_fn(|r#fn|
					r#fn.with_self(MutTakeSelf)
						.with_param(rhs_bind.clone(), *rhs_type)
						.return_type(*output)
						.body(format!("{assign_fn}(&mut {lhs}, rhs); {ret}"))
				);
		}

		self.push(assign_data);
		for Binary { target, lhs, lhs_mut, rhs_bind, rhs_type, .. } in ops {
			let lhs: Cow<_> = if *lhs_mut {
				format!("&mut {lhs}").into()
			} else {
				(*lhs).into()
			};

			self.push_impl(target, Some(rhs_type))
				.push_op_fn(|r#fn|
					r#fn.with_self(MutSelf)
						.with_param(rhs_bind.clone(), *rhs_type)
						.body(format!("{assign_fn}({lhs}, rhs);"))
				);
		}
		self
	}

	pub fn unary(&mut self, data: &'a dyn OpData, target: &str) {
		self.push(data)
			.push_impl(target, None)
			.output_type("Self")
			.push_op_fn(|r#fn|
				r#fn.with_self(MutTakeSelf)
					.return_type("Self")
					.body(
						format!(
							"self.0 = {}::{}(self.0); self",
							data.trait_name(),
							data.fn_name()
						)
					)
			);
	}

	pub fn comparing(&mut self, data: &'a dyn OpData, ret_type: &str, ops: impl IntoIterator<Item = Comparing<'a>>) {
		self.push(data);
		for Comparing { target, other_bind, other_type, body } in ops {
			self.push_impl(target, Some(other_type))
				.push_op_fn(|r#fn|
					r#fn.with_self(RefSelf)
						.with_param(other_bind, format!("&{other_type}"))
						.return_type(ret_type)
						.body(body)
				);
		}
	}

	pub fn formatting<O: OpData>(&mut self, target: &str, ops: &'a [O]) {
		for op in ops {
			self.push(op)
				.push_impl(target, None)
				.push_op_fn(|r#fn|
					r#fn.with_self(RefSelf)
						.with_param("f", "&mut core::fmt::Formatter<'_>")
						.return_type("core::fmt::Result")
						.body(format!("{}::fmt(&self.0, f)", op.trait_name()))
				);
		}
	}

	pub fn accumulating(&mut self, data: &'a dyn OpData, ops: &[Accumulating]) {
		self.push(data);
		for Accumulating { target, element_type, body } in ops {
			self.push_impl(target, Some(element_type))
				.push_op_fn(|r#fn|
					r#fn.generic_type("I", format!("Iterator<Item = {element_type}>"))
						.with_param("iter", "I")
						.return_type("Self")
						.body(body.to_string())
				);
		}
	}

	fn push(&mut self, op: &'a dyn OpData) -> &mut Self {
		self.op = Some(op);
		self
	}

	fn push_impl(&mut self, target: &str, generic_param: Option<&str>) -> Impl<'_> {
		let name = self.cur().trait_name();
		let generics = generic_param.as_slice().iter().cloned();
		let mut builder = self.gen
							  .impl_trait_for_other_type(name, target)
							  .with_trait_generics(generics);
		builder.impl_outer_attr("automatically_derived").unwrap();
		Impl { fn_name: self.op.unwrap().fn_name(), builder }
	}

	pub fn finish(self) -> Result<TokenStream> {
		self.gen.finish()
	}
}

impl Sink<'_> {
	fn cur(&self) -> &dyn OpData {
		self.op.unwrap()
	}
}

impl<'a> Impl<'a> {
	fn output_type(mut self, output: &str) -> Self {
		self.builder
			.impl_type("Output", output)
			.expect("invalid output type");
		self
	}

	fn push_op_fn(self, build: impl FnOnce(Fn<'a, '_>)) -> Self {
		let name = self.fn_name;
		self.push_fn(name, build)
	}

	fn push_fn(mut self, name: &str, build: impl FnOnce(Fn<'a, '_>)) -> Self {
		build(Fn(self.builder.generate_fn(name)));
		self
	}
}

impl<'a> Fn<'a, '_> {
	fn with_self(mut self, self_arg: FnSelfArg) -> Self {
		self.0 = self.0.with_self_arg(self_arg);
		self
	}

	fn with_param(mut self, name: impl Into<String>, ty: impl Into<String>) -> Self {
		self.0 = self.0.with_arg(name, ty);
		self
	}

	fn return_type(mut self, ty: impl Into<String>) -> Self {
		self.0 = self.0.with_return_type(ty);
		self
	}

	fn generic_type(mut self, name: impl Into<String>, bound: impl Into<String>) -> Self {
		self.0 = self.0.with_generic_deps(name, [bound]);
		self
	}

	fn body(self, body: impl AsRef<str>) {
		self.0
			.body(|bb| {
				bb.push_parsed(body)?;
				Ok(())
			})
			.expect("invalid body");
	}
}
