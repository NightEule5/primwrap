// SPDX-License-Identifier: Apache-2.0

use std::borrow::Cow;
use FnSelfArg::{TakeSelf, MutSelf};
use virtue::prelude::*;
use virtue::prelude::FnSelfArg::RefSelf;
use crate::ops::OpData;

pub struct ImplSink<'a> {
	gen: &'a mut Generator,
	base_trait_name: &'static str,
	assign_trait_name: Option<&'static str>,
	base_fn_name: &'static str,
	assign_fn_name: Option<&'static str>,
}

pub struct Impl<'a> {
	target: &'a str,
	output: Option<&'a str>,
	self_arg: FnSelfArg,
	mut_self: bool,
	param: Option<(Cow<'a, str>, Cow<'a, str>)>,
	has_generic: bool,
	ret_type: Option<&'a str>,
	body: Cow<'a, str>
}

pub enum ImplSpec<'a> {
	Binary {
		target: &'a str,
		output: Option<&'a str>,
		operand_bind: Cow<'a, str>,
		operand_type: &'a str,
		body: String
	},
	Assign {
		target: &'a str,
		operand_bind: Cow<'a, str>,
		operand_type: &'a str,
		expr: String
	},
	Unary {
		target: &'a str,
		body: String
	},
	Comparison {
		target: &'a str,
		other_type: &'a str,
		ret_type: &'a str,
		body: &'a str
	},
	Format {
		target: &'a str,
		body: &'a str
	}
}

impl<'a> ImplSink<'a> {
	pub fn new_for(gen: &'a mut Generator, op: &impl OpData) -> Self {
		Self::new(
			gen,
			op.trait_name(),
			op.assign_trait_name(),
			op.fn_name(),
			op.assign_fn_name()
		)
	}

	pub fn new(
		gen: &'a mut Generator,
		base_trait_name: &'static str,
		assign_trait_name: Option<&'static str>,
		base_fn_name: &'static str,
		assign_fn_name: Option<&'static str>,
	) -> Self {
		Self {
			gen,
			base_trait_name,
			assign_trait_name,
			base_fn_name,
			assign_fn_name,
		}
	}

	pub fn push(&mut self, is: ImplSpec<'_>) -> Result {
		let (trait_name, fn_name) = match &is {
			ImplSpec::Binary { .. } |
			ImplSpec::Unary  { .. } =>
				(self.base_trait_name, self.base_fn_name),
			ImplSpec::Assign { .. } =>
				(self.assign_trait_name.unwrap(), self.assign_fn_name.unwrap()),
			_ => unreachable!()
		};
		Impl::from(is).build(self.gen, trait_name, fn_name)
	}
}

impl<'a> From<ImplSpec<'a>> for Impl<'a> {
	fn from(value: ImplSpec<'a>) -> Self {
		let (target, output, self_arg, mut_self, param, has_generic, ret_type, body) = match value {
			ImplSpec::Binary { target, output, operand_bind, operand_type, body } => (
				target,
				output,
				TakeSelf,
				true,
				Some((operand_bind.into(), operand_type.into())),
				true,
				output,
				body.into(),
			),
			ImplSpec::Assign { target, operand_bind, operand_type, expr } => (
				target,
				None,
				MutSelf,
				false,
				Some((operand_bind.into(), operand_type.into())),
				true,
				None,
				expr.into()
			),
			ImplSpec::Unary { target, body } => (
				target,
				Some("Self"),
				TakeSelf,
				false,
				None,
				true,
				Some("Self"),
				body.into()
			),
			ImplSpec::Comparison { target, other_type, ret_type, body } => (
				target,
				None,
				RefSelf,
				false,
				Some(("other".into(), format!("&{other_type}").into())),
				true,
				Some(ret_type),
				body.into()
			),
			ImplSpec::Format { target, body } => (
				target,
				None,
				RefSelf,
				false,
				Some(("f".into(), "&mut core::fmt::Formatter<'_>".into())),
				false,
				Some("core::fmt::Result"),
				body.into()
			)
		};

		Self {
			target,
			output,
			self_arg,
			mut_self,
			param,
			has_generic,
			ret_type,
			body
		}
	}
}

impl Impl<'_> {
	pub fn build(self, gen: &mut Generator, trait_name: &str, trait_fn_name: &str) -> Result {
		let mut gen = gen.impl_trait_for_other_type(
			self.format_trait(trait_name),
			self.target
		);
		gen.impl_outer_attr("automatically_derived")?;

		if let Some(output) = self.output {
			gen.impl_type("Output", output)?;
		}

		let mut gen = gen.generate_fn(trait_fn_name);
		gen = match (self.self_arg, self.mut_self) {
			(TakeSelf, true) => gen.with_arg("mut self", "Self"),
			(self_arg, _   ) => gen.with_self_arg(self_arg)
		};
		if let Some((param_bind, param_type)) = self.param {
			gen = gen.with_arg(param_bind.into_owned(), param_type.into_owned());
		}
		if let Some(rt) = self.ret_type {
			gen = gen.with_return_type(rt);
		}

		gen.body(|b| {
			b.push_parsed(self.body)?;
			Ok(())
		})
	}

	fn format_trait(&self, trait_name: &str) -> String {
		self.param
			.as_ref()
			.filter(|(_, ty)| self.has_generic && ty != "Self")
			.map_or_else(
				|| trait_name.into(),
				|(_, ty)|
					format!(
						"{trait_name}<{}>",
						ty.trim_start_matches('&')
					)
			)
	}
}
