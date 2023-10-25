// SPDX-License-Identifier: Apache-2.0

use virtue::prelude::*;
use crate::util::Impl;
use crate::util::ImplSpec::Format;
use const_format::concatcp;

pub fn generate_fmt(gen: &mut Generator, target: &str, inner: &str) -> Result {
	macro_rules! gen {
		($($fmt_trait:ident),+) => {
			$(
			#[allow(non_snake_case)]
			let $fmt_trait = |gen| {
				const PATH: &str = concat!("core::fmt::", stringify!($fmt_trait));
				const BODY: &str = concatcp!(PATH, "::fmt(&self.0, f)");
				Impl::from(Format { target, body: BODY }).build(gen, PATH, "fmt")
			};
			)+
		};
	}

	macro_rules! build {
		($($fmt_trait:ident),+) => {{
			$($fmt_trait(gen)?;)+
		}};
	}

	gen!(Debug, Display, Binary, Octal, LowerExp, LowerHex, UpperExp, UpperHex);

	build!(Debug, Display);
	match inner {
		"bool" => { }
		_ if inner.starts_with('f') => build!(LowerExp, UpperExp),
		_ => build!(Binary, Octal, LowerExp, LowerHex, UpperExp, UpperHex)
	}

	Ok(())
}
