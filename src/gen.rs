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
		for op in ops {
			let &Binary { target, output, lhs, rhs_type, .. } = op;
			let Binary { rhs_bind, base_ret, .. } = op;
			let ret: Cow<_> = if let Some(ret) = base_ret {
				ret.to_string().into()
			} else {
				"self".into()
			};

			self.push_impl(target, Some(rhs_type))
				.output_type(output)
				.push_op_fn(|r#fn|
					r#fn.with_self_arg(MutTakeSelf)
						.with_arg(rhs_bind.clone(), rhs_type)
						.with_return_type(output)
						.parsed_body(format!("{assign_fn}(&mut {lhs}, rhs); {ret}"))
				);
		}

		self.push(assign_data);
		for op in ops {
			let &Binary { target, lhs, lhs_mut, rhs_type, .. } = op;
			let lhs: Cow<_> = if lhs_mut {
				format!("&mut {lhs}").into()
			} else {
				(*lhs).into()
			};

			self.push_impl(target, Some(rhs_type))
				.push_op_fn(|r#fn|
					r#fn.with_self_arg(MutSelf)
						.with_arg(op.rhs_bind.clone(), rhs_type)
						.parsed_body(format!("{assign_fn}({lhs}, rhs);"))
				);
		}
		self
	}

	pub fn unary(&mut self, data: &'a dyn OpData, target: &str) {
		self.push(data)
			.push_impl(target, None)
			.output_type("Self")
			.push_op_fn(|r#fn|
				r#fn.with_self_arg(MutTakeSelf)
					.with_return_type("Self")
					.parsed_body(
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
					r#fn.with_self_arg(RefSelf)
						.with_arg(other_bind, format!("&{other_type}"))
						.with_return_type(ret_type)
						.parsed_body(body)
				);
		}
	}

	pub fn formatting<O: OpData>(&mut self, target: &str, ops: &'a [O]) {
		for op in ops {
			let body = format!("{}::fmt(&self.0, f)", op.trait_name());
			self.push(op)
				.push_impl(target, None)
				.push_op_fn(|r#fn|
					r#fn.with_self_arg(RefSelf)
						.with_arg("f", "&mut core::fmt::Formatter<'_>")
						.with_return_type("core::fmt::Result")
						.parsed_body(body)
				);
		}
	}

	pub fn accumulating(&mut self, data: &'a dyn OpData, ops: &[Accumulating]) {
		self.push(data);
		for op in ops {
			let &Accumulating { target, element_type, .. } = op;
			self.push_impl(target, Some(element_type))
				.push_op_fn(|r#fn|
					r#fn.with_generic_deps("I", [format!("Iterator<Item = {element_type}>")])
						.with_arg("iter", "I")
						.with_return_type("Self")
						.parsed_body(op.body.clone())
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

	fn push_op_fn(self, build: impl FnOnce(FnBuilder<'_, ImplFor<'a, Generator>>)) -> Self {
		let name = self.fn_name;
		self.push_fn(name, build)
	}

	fn push_fn(mut self, name: &str, build: impl FnOnce(FnBuilder<'_, ImplFor<'a, Generator>>)) -> Self {
		build(self.builder.generate_fn(name));
		self
	}
}

trait FnBuilderExt: Sized {
	fn parsed_body(self, body: impl AsRef<str>);
}

impl FnBuilderExt for FnBuilder<'_, ImplFor<'_, Generator>> {
	fn parsed_body(self, body: impl AsRef<str>) {
		self.body(|bb| {
			bb.push_parsed(body)?;
			Ok(())
		}).expect("invalid body");
	}
}
