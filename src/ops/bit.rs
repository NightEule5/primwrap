// SPDX-License-Identifier: Apache-2.0

use crate::gen::Sink;
use super::{Binary, Op, op_enum};

op_enum! {
	enum Bit in ops {
		BitAnd = "bitand" with_assign,
		BitOr  = "bitor"  with_assign,
		BitXor = "bitxor" with_assign,
		Shl    = "shl"    with_assign,
		Shr    = "shr"    with_assign,
		Not    = "not"
	}
}

impl Op for Bit {
	fn supported(type_name: &str) -> &[Self] {
		match type_name {
			"bool" => &[Self::BitAnd, Self::BitOr, Self::BitXor, Self::Not],
			name if name.starts_with('f') => &[],
			_ => &[Self::BitAnd, Self::BitOr, Self::BitXor, Self::Shl, Self::Shr, Self::Not]
		}
	}

	fn generate<'a>(&'a self, sink: &mut Sink<'a>, target: &'a str, inner: &'a str) {
		if let Self::Not = self {
			sink.unary(self, target);
		} else if let Self::Shl | Self::Shr = self {
			static INT_TYPES: &[&str] = &[
				"u8", "u16", "u32", "u64", "u128", "usize",
				"i8", "i16", "i32", "i64", "i128", "isize"
			];

			if INT_TYPES.contains(&inner) {
				sink.binary(self, &[Binary::self_self(target)]);
			} else {
				sink.binary(self, &Binary::permutations(target, inner));
			}

			for &int_type in INT_TYPES {
				if int_type == inner {
					continue
				}

				sink.binary(self, &Binary::inner_permutations(target, inner, int_type));
			}
		} else {
			sink.binary(self, &Binary::permutations(target, inner));
		}
	}
}
