// SPDX-License-Identifier: Apache-2.0

use virtue::prelude::*;
use crate::util::Impl;
use crate::util::ImplSpec::Accum;

pub fn generate_accum(gen: &mut Generator, has_arith: bool, target: &str, inner: &str) -> Result {
	if !has_arith || inner == "bool" {
		return Ok(())
	}

	const SUM:     &str = "core::iter::Sum";
	const PRODUCT: &str = "core::iter::Product";

	let mut build = |trait_path, fn_name| {
		Impl::from(Accum { target, element_type: "Self", body: format!("iter.map(|it| it.0).{fn_name}()") })
			.build(gen, trait_path, fn_name)?;
		Impl::from(Accum { target, element_type: inner, body: format!("Self(iter.{fn_name}())") })
			.build(gen, &format!("{trait_path}<{inner}>"), fn_name)?;
		Impl::from(Accum { target: inner, element_type: target, body: format!("iter.{fn_name}::<{target}>().0") })
			.build(gen, &format!("{trait_path}<{target}>"), fn_name)?;
		Ok(())
	};

	build(SUM,     "sum"    )?;
	build(PRODUCT, "product")
}
