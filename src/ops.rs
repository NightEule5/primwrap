// SPDX-License-Identifier: Apache-2.0

use virtue::Error;
use virtue::generate::{FnSelfArg, Generator};

const B: i32 = 0;
const U: i32 = 1;

const ARITH: &[&dyn Operation] = &[
	&Op::<"Add", "add", B>,
	&Op::<"Sub", "sub", B>,
	&Op::<"Mul", "mul", B>,
	&Op::<"Div", "div", B>,
	&Op::<"Rem", "rem", B>,
	&Op::<"Neg", "neg", U>,
];

pub fn arithmetic_ops_for_type(type_name: &str) -> &[&dyn Operation] {
	// Exclude negation for unsigned types
	if type_name.starts_with('u') {
		&ARITH[..5]
	} else {
		ARITH
	}
}

impl BinaryOperation for Op<"Add", "add", B> { }
impl BinaryOperation for Op<"Sub", "sub", B> { }
impl BinaryOperation for Op<"Mul", "mul", B> { }
impl BinaryOperation for Op<"Div", "div", B> { }
impl BinaryOperation for Op<"Rem", "rem", B> { }
impl  UnaryOperation for Op<"Neg", "neg", U> { }

pub struct Op<const TRAIT: &'static str, const METHOD: &'static str, const TYPE: i32>;

pub trait Operation {
	fn generate_impl(&self, gen: &mut Generator, self_type: &str, inner_type: &str) -> Result<(), Error>;
}

trait BaseOperation {
	fn trait_name(&self) -> String;
	fn method(&self) -> &str;
}

#[derive(Copy, Clone)]
enum BinaryType {
	SelfSelf,
	SelfOperand,
	OperandSelf
}

trait BinaryOperation: BaseOperation {
	fn gen_binary(&self, gen: &mut Generator, operand: &str, output: &str, ty: BinaryType) -> Result<(), Error> {
		let trait_name = format!("{}<{operand}>", self.trait_name());
		let param = match ty {
			BinaryType::SelfSelf => "Self(rhs)".into(),
			BinaryType::OperandSelf => format!("{operand}(rhs)"),
			_ => "rhs".into()
		};
		let expr = self.expr(ty);

		self.gen_assign(gen, operand, &param, output, &expr)?;

		// Use impl_trait_for_other_type instead of impl_for for greater flexibility.
		// It's all the same underneath anyway.
		let mut r#if = gen.impl_trait_for_other_type(trait_name, output);
		r#if.impl_outer_attr("automatically_derived")?;
		r#if.impl_type("Output", output)?;
		r#if.generate_fn(self.method())
			.with_arg("mut self", "Self")
			.with_arg(param, operand)
			.with_return_type(output)
			.body(|body| {
				body.push_parsed(expr)?.ident_str("self");
				Ok(())
			})?;
		drop(r#if);

		// Generate the inverse (impl Op<Wrapper> for Primitive).
		if let BinaryType::SelfOperand = ty {
			self.gen_binary(gen, output, operand, BinaryType::OperandSelf)?;
		}

		Ok(())
	}

	fn gen_assign(&self, gen: &mut Generator, operand: &str, operand_param: &str, output: &str, expr: &str) -> Result<(), Error> {
		let trait_name = format!("{}Assign<{operand}>", self.trait_name());
		let mut r#if = gen.impl_trait_for_other_type(trait_name, output);
		r#if.impl_outer_attr("automatically_derived")?;
		r#if.generate_fn(format!("{}_assign", self.method()))
			.with_self_arg(FnSelfArg::MutSelf)
			.with_arg(operand_param, operand)
			.body(|body| {
				body.push_parsed(expr)?;
				Ok(())
			})
	}

	fn expr(&self, ty: BinaryType) -> String {
		let method = self.method();
		let trait_name = self.trait_name();
		let field_access = match ty {
			BinaryType::SelfSelf |
			BinaryType::SelfOperand => ".0",
			BinaryType::OperandSelf => ""
		};
		format!(
			"use {trait_name}Assign;\
			self{field_access}.{method}_assign(rhs);"
		)
	}
}

trait UnaryOperation: BaseOperation {
	fn generate_unary_impl(&self, gen: &mut Generator) -> Result<(), Error> {
		let trait_name = self.trait_name();
		let method = self.method();
		let mut r#if = gen.impl_for(trait_name);
		r#if.impl_outer_attr("automatically_derived")?;
		r#if.impl_type("Output", "Self")?;
		r#if.generate_fn(self.method())
			.with_self_arg(FnSelfArg::TakeSelf)
			.with_return_type("Self")
			.body(|body| {
				body.push_parsed(format!("Self(self.0.{method}())"))?;
				Ok(())
			})
	}
}

impl<const T: &'static str, const M: &'static str, const TY: i32> BaseOperation for Op<T, M, TY> {
	fn trait_name(&self) -> String { format!("core::ops::{}", T) }

	fn method(&self) -> &str { M }
}

impl<const T: &'static str, const M: &'static str> Operation for Op<T, M, B>
where Self: BinaryOperation {
	fn generate_impl(&self, gen: &mut Generator, self_type: &str, inner_type: &str) -> Result<(), Error> {
		self.gen_binary(gen, self_type, self_type, BinaryType::SelfSelf)?;
		self.gen_binary(gen, inner_type, self_type, BinaryType::SelfOperand)
	}
}

impl<const T: &'static str, const M: &'static str> Operation for Op<T, M, U>
where Self: UnaryOperation {
	fn generate_impl(&self, gen: &mut Generator, _: &str, _: &str) -> Result<(), Error> {
		self.generate_unary_impl(gen)
	}
}
