// SPDX-License-Identifier: Apache-2.0

use crate::gen::Sink;
use super::{Binary, Op, op_enum};

op_enum! {
	enum Arithmetic in ops {
		Add = "add" with_assign,
		Sub = "sub" with_assign,
		Mul = "mul" with_assign,
		Div = "div" with_assign,
		Rem = "rem" with_assign,
		Neg = "neg"
	}
}

impl Op for Arithmetic {
	fn supported(type_name: &str) -> &[Self] {
		match type_name {
			"bool" => &[],
			name if name.starts_with('u') => &Self::VARS[..5],
			_ => Self::VARS
		}
	}

	fn generate<'a>(&'a self, sink: &mut Sink<'a>, target: &'a str, inner: &'a str) {
		if let Self::Neg = self {
			sink.unary(self, target);
		} else {
			sink.binary(self, &Binary::permutations(target, inner));
		}
	}
}
