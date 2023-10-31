// SPDX-License-Identifier: Apache-2.0

use crate::gen::Sink;
use super::{Op, op_enum};

op_enum! {
	enum Formatting in fmt {
		Debug = "fmt",
		Display,
		Binary,
		Octal,
		LowerExp,
		LowerHex,
		UpperExp,
		UpperHex
	}
}

impl Op for Formatting {
	fn supported(type_name: &str) -> &[Self] {
		match type_name {
			"bool" => &Self::VARS[..2],
			_ if type_name.starts_with('f') =>
				&[Self::Debug, Self::Display, Self::LowerExp, Self::UpperExp],
			_ => Self::VARS
		}
	}

	fn generate_all<'a>(sink: &mut Sink<'a>, target: &'a str, inner: &'a str) where Self: 'a {
		sink.formatting(target, Self::supported(inner));
	}

	fn generate<'a>(&'a self, _sink: &mut Sink<'a>, _target: &'a str, _inner: &'a str) {
		unimplemented!()
	}
}
