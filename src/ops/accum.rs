// SPDX-License-Identifier: Apache-2.0

use crate::gen::Sink;
use crate::ops::{Accumulating, Op, op_enum, OpData};

op_enum! {
	enum Accumulation in iter {
		Sum = "sum",
		Product = "product"
	}
}

impl Op for Accumulation {
	fn supported(type_name: &str) -> &[Self] {
		if type_name == "bool" {
			&[]
		} else {
			Self::VARS
		}
	}

	fn generate<'a>(&'a self, sink: &mut Sink<'a>, target: &'a str, inner: &'a str) {
		sink.accumulating(self, &[
			Accumulating::self_self(target, self.fn_name()),
			Accumulating::self_inner(target, inner, self.fn_name()),
			Accumulating::inner_self(target, inner, self.fn_name())
		])
	}
}
