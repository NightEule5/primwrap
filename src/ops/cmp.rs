// SPDX-License-Identifier: Apache-2.0

use crate::gen::Sink;
use super::{Comparing, Op, op_enum};

macro_rules! cmp_enum {
	(enum $name:ident { $($op_name:ident = $fn_name:literal),+ }) => {
		op_enum! {
			enum $name {
				$($op_name = $fn_name),+
			}
		}
		
		impl $name {
			fn impl_body(&self, swapped: bool) -> &'static str {
				match (self, swapped) {
					$((Self::$op_name, false) => concat!("self.0.", $fn_name, "(other)"),)+
					$((Self::$op_name, true) => concat!("self.", $fn_name, "(other)")),+
				}
			}
		}
	};
}

cmp_enum! {
	enum Comparison {
		PartialEq  = "eq",
		PartialOrd = "partial_cmp"
	}
}

impl Comparison {
	fn ret_type(&self) -> &'static str {
		match self {
			Self::PartialEq  => "bool",
			Self::PartialOrd => "Option<core::cmp::Ordering>"
		}
	}
}

impl Op for Comparison {
	fn supported(_: &str) -> &[Self] { Self::VARS }

	fn generate<'a>(&'a self, sink: &mut Sink<'a>, target: &'a str, inner: &'a str) {
		sink.comparing(self, self.ret_type(), [
			Comparing {
				target,
				other_bind: "other".into(),
				other_type: inner,
				body: self.impl_body(false),
			},
			Comparing {
				target: inner,
				other_bind: format!("{target}(other)").into(),
				other_type: target,
				body: self.impl_body(true),
			}
		])
	}
}
