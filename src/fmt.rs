// SPDX-License-Identifier: Apache-2.0

use virtue::prelude::*;
use crate::util::Impl;
use crate::util::ImplSpec::Format;

macro_rules! body {
    ($fmt_trait:literal) => {
		concat!("core::fmt::", $fmt_trait, "::fmt(&self.0, f)")
	};
}

pub fn generate_fmt(gen: &mut Generator, target: &str, inner: &str) -> Result {
	macro_rules! gen_fmt {
		($($fmt_trait:literal),+) => {
			$(
			Impl::from(
				Format { target, body: body!($fmt_trait) }
			).build(gen, concat!("core::fmt::", $fmt_trait), "fmt")?;
			)+
		};
	}

	gen_fmt!("Debug", "Display");

	if inner != "bool" {
		gen_fmt!("Octal", "Binary", "LowerHex", "UpperHex", "LowerExp", "UpperExp");
	}

	Ok(())
}
