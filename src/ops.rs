// SPDX-License-Identifier: Apache-2.0

use strum::{EnumIter, IntoEnumIterator, IntoStaticStr};
use virtue::generate::{FnSelfArg, Generator};
use virtue::prelude::*;

pub trait Op: Sized {
	fn trait_name(&self) -> &'static str;
	fn fn_name(&self) -> &'static str;

	fn expand(&self, gen: &mut Generator, target: &str, inner: &str) -> Result;

	fn expand_as_binary(&self, gen: &mut Generator, target: &str, inner: &str) -> Result {
		expand_binary(self, gen, "Self", target, target, BinaryType::SelfSelf)?;
		expand_binary(self, gen, inner, target, target, BinaryType::SelfOperand)
	}
}

pub fn arithmetic_ops_for_type(type_name: &str) -> impl Iterator<Item = Arithmetic> {
	let iter = Arithmetic::iter();

	// Exclude negation for unsigned types
	if type_name.starts_with('u') {
		iter.take(5)
	} else {
		iter.take(6)
	}
}

#[derive(Clone, Copy, EnumIter, IntoStaticStr)]
pub enum Arithmetic {
	Add,
	Sub,
	Mul,
	Div,
	Rem,
	Neg
}

impl Op for Arithmetic {
	fn trait_name(&self) -> &'static str {
		self.into()
	}

	fn fn_name(&self) -> &'static str {
		match self {
			Self::Add => "add",
			Self::Sub => "sub",
			Self::Mul => "mul",
			Self::Div => "div",
			Self::Rem => "rem",
			Self::Neg => "neg"
		}
	}

	fn expand(&self, gen: &mut Generator, target: &str, inner: &str) -> Result {
		if let Self::Neg = self {
			expand_unary(self, gen)
		} else {
			self.expand_as_binary(gen, target, inner)
		}
	}
}

#[derive(Copy, Clone)]
enum BinaryType {
	SelfSelf,
	SelfOperand,
	OperandSelf
}

fn expand_binary(op: &impl Op, gen: &mut Generator, operand: &str, output: &str, target: &str, ty: BinaryType) -> Result {
	let trait_name = format!("core::ops::{}<{operand}>", op.trait_name());
	let param = match ty {
		BinaryType::SelfSelf => "Self(rhs)".into(),
		BinaryType::OperandSelf => format!("{operand}(rhs)"),
		_ => "rhs".into()
	};
	let expr = binary_expr(op, ty);

	expand_assign(op, gen, operand, &param, target, &expr)?;

	// Use impl_trait_for_other_type instead of impl_for for greater flexibility.
	// It's all the same underneath anyway.
	let mut r#if = gen.impl_trait_for_other_type(trait_name, target);
	r#if.impl_outer_attr("automatically_derived")?;
	r#if.impl_type("Output", output)?;
	r#if.generate_fn(op.fn_name())
		.with_arg("mut self", "Self")
		.with_arg(param, operand)
		.with_return_type(output)
		.body(|body| {
			body.push_parsed(expr)?;

			if let BinaryType::OperandSelf = ty {
				// For inverse operations, wrap the result at the end.
				body.push_parsed(format!("{output}(self)"))?;
			} else {
				body.ident_str("self");
			}

			Ok(())
		})?;
	drop(r#if);

	// Generate the inverse (impl Op<Wrapper> for Primitive).
	if let BinaryType::SelfOperand = ty {
		expand_binary(op, gen, target, target, operand, BinaryType::OperandSelf)?;
	}

	Ok(())
}

fn expand_assign(op: &impl Op, gen: &mut Generator, operand: &str, operand_param: &str, output: &str, expr: &str) -> Result {
	let trait_name = format!("core::ops::{}Assign<{operand}>", op.trait_name());
	let mut r#if = gen.impl_trait_for_other_type(trait_name, output);
	r#if.impl_outer_attr("automatically_derived")?;
	r#if.generate_fn(format!("{}_assign", op.fn_name()))
		.with_self_arg(FnSelfArg::MutSelf)
		.with_arg(operand_param, operand)
		.body(|body| {
			body.push_parsed(expr)?;
			Ok(())
		})
}

fn binary_expr(op: &impl Op, ty: BinaryType) -> String {
	let fn_name = op.fn_name();
	let trait_name = op.trait_name();
	let field_access = match ty {
		BinaryType::SelfSelf |
		BinaryType::SelfOperand => ".0",
		BinaryType::OperandSelf => ""
	};
	format!(
		"use core::ops::{trait_name}Assign;\
		self{field_access}.{fn_name}_assign(rhs);"
	)
}

fn expand_unary(op: &impl Op, gen: &mut Generator) -> Result {
	let trait_name = op.trait_name();
	let fn_name = op.fn_name();
	let mut r#if = gen.impl_for(trait_name);
	r#if.impl_outer_attr("automatically_derived")?;
	r#if.impl_type("Output", "Self")?;
	r#if.generate_fn(fn_name)
		.with_self_arg(FnSelfArg::TakeSelf)
		.with_return_type("Self")
		.body(|body| {
			body.push_parsed(format!("Self(self.0.{fn_name}())"))?;
			Ok(())
		})
}
