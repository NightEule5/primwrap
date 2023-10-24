// SPDX-License-Identifier: Apache-2.0

use virtue::prelude::*;
use crate::util::{ImplSink, ImplSpec};
use crate::util::ImplSpec::*;

macro_rules! binary_op_filter {
	(binary $($concat:tt)+) => {
		Some($($concat)+)
	};
	(unary $($concat:tt)+) => {
		None
	}
}

macro_rules! op_enum {
	(enum $name:ident { $($op_type:ident $op_name:ident = $fn_name:literal),+ }) => {
		#[derive(Clone, Copy)]
		pub enum $name {
			$($op_name),+
		}

		impl OpData for $name {
			fn trait_name(&self) -> &'static str {
				match self {
					$(Self::$op_name => stringify!(core::ops::$op_name)),+
				}
			}

			fn fn_name(&self) -> &'static str {
				match self {
					$(Self::$op_name => $fn_name),+
				}
			}

			fn assign_trait_name(&self) -> Option<&'static str> {
				match self {
					$(Self::$op_name => binary_op_filter!($op_type concat!(stringify!(core::ops::$op_name), "Assign"))),+
				}
			}

			fn assign_fn_name(&self) -> Option<&'static str> {
				match self {
					$(Self::$op_name => binary_op_filter!($op_type concat!($fn_name, "_assign"))),+
				}
			}
		}
	};
}

op_enum! {
	enum Arithmetic {
		binary Add = "add",
		binary Sub = "sub",
		binary Mul = "mul",
		binary Div = "div",
		binary Rem = "rem",
		 unary Neg = "neg"
	}
}

op_enum! {
	enum Bit {
		binary BitAnd = "bitand",
		binary BitOr  = "bitor",
		binary BitXor = "bitxor",
		binary Shl    = "shl",
		binary Shr    = "shr",
		 unary Not    = "not"
	}
}

pub trait OpData {
	fn trait_name(&self) -> &'static str;
	fn fn_name(&self) -> &'static str;
	fn assign_trait_name(&self) -> Option<&'static str>;
	fn assign_fn_name(&self) -> Option<&'static str>;
}

pub trait Op: OpData + Sized {
	fn supported(type_name: &str) -> &[Self];

	fn generate_all(gen: &mut Generator, target: &str, inner: &str) -> Result {
		for op in Self::supported(inner) {
			let sink = ImplSink::new_for(gen, op);
			op.generate(sink, target, inner)?;
		}
		Ok(())
	}

	fn generate(&self, sink: ImplSink, target: &str, inner: &str) -> Result;

	fn generate_as_binary_with_self(&self, sink: &mut ImplSink, target: &str) -> Result {
		self.generate_as_binary(
			sink,
			target,
			Some("Self(rhs)".into()),
			"Self",
			"Self",
			format!(
				"{}::{}(&mut self.0, rhs)",
				self.assign_trait_name().unwrap(),
				self.assign_fn_name().unwrap()
			),
			None
		)
	}

	fn generate_as_binary_with_inverse(&self, sink: &mut ImplSink, target: &str, operand: &str, inner: &str) -> Result {
		let trait_name = self.assign_trait_name().unwrap();
		let fn_name = self.assign_fn_name().unwrap();
		let expr = |field_access|
			format!(
				"use {trait_name};\
				self{field_access}.{fn_name}(rhs)"
			);

		self.generate_as_binary(
			sink,
			target,
			None,
			operand,
			"Self",
			expr(".0"),
			None
		)?;

		let cast = if inner == operand {
			format!("")
		} else {
			format!(" as {inner}")
		};

		self.generate_as_binary(
			sink,
			operand,
			Some(format!("{target}(rhs)")),
			target,
			target,
			expr(""),
			Some(format!("{target}(self{cast})"))
		)
	}

	fn generate_as_binary(
		&self,
		sink: &mut ImplSink,
		target: &str,
		operand_bind: Option<String>,
		operand_type: &str,
		output: &str,
		expr: String,
		ret: Option<String>
	) -> Result {
		let ret = ret.unwrap_or_else(|| "self".into());

		sink.push(Binary {
			target,
			output: Some(output),
			operand_bind: operand_bind.clone().map_or("rhs".into(), Into::into),
			operand_type,
			body: format!("{expr}; {ret}")
		})?;
		sink.push(Assign {
			target,
			operand_bind: operand_bind.map_or("rhs".into(), Into::into),
			operand_type,
			expr
		})
	}

	fn generate_as_unary<'a>(&'a self, target: &'a str) -> ImplSpec<'a> {
		Unary {
			target,
			body: format!(
				"Self({}::{}(self.0))",
				self.trait_name(),
				self.fn_name()
			)
		}
	}
}

impl Op for Arithmetic {
	fn supported(type_name: &str) -> &[Self] {
		let ops: &[Self] = &[
			Self::Add,
			Self::Sub,
			Self::Mul,
			Self::Div,
			Self::Rem,
			Self::Neg
		];

		match type_name {
			"bool" => &[],
			name if name.starts_with('u') => &ops[..5],
			_ => ops
		}
	}

	fn generate(&self, ref mut sink: ImplSink, target: &str, inner: &str) -> Result {
		if let Self::Neg = self {
			sink.push(self.generate_as_unary(target))
		} else {
			self.generate_as_binary_with_self(sink, target)?;
			self.generate_as_binary_with_inverse(sink, target, inner, inner)
		}
	}
}

impl Op for Bit {
	fn supported(type_name: &str) -> &[Self] {
		if type_name == "bool" {
			&[Self::BitAnd, Self::BitOr, Self::BitXor, Self::Not]
		} else {
			&[Self::BitAnd, Self::BitOr, Self::BitXor, Self::Shl, Self::Shr, Self::Not]
		}
	}

	fn generate(&self, ref mut sink: ImplSink, target: &str, inner: &str) -> Result {
		if let Self::Not = self {
			sink.push(self.generate_as_unary(target))
		} else if let Self::Shl | Self::Shr = self {
			static INT_TYPES: &[&str] = &[
				"u8", "u16", "u32", "u64", "u128", "usize",
				"i8", "i16", "i32", "i64", "i128", "isize"
			];

			self.generate_as_binary_with_self(sink, target)?;
			if !INT_TYPES.contains(&inner) {
				self.generate_as_binary_with_inverse(sink, target, inner, inner)?;
			}

			for &int_type in INT_TYPES {
				self.generate_as_binary_with_inverse(sink, target, int_type, inner)?;
			}
			
			Ok(())
		} else {
			self.generate_as_binary_with_self(sink, target)?;
			self.generate_as_binary_with_inverse(sink, target, inner, inner)
		}
	}
}
