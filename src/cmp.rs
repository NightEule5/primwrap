// SPDX-License-Identifier: Apache-2.0

use virtue::prelude::*;
use crate::util::Impl;
use crate::util::ImplSpec::Comparison;

pub fn generate_cmp(gen: &mut Generator, target: &str, other: &str) -> Result {
	macro_rules! gen_cmp {
		($trait:literal, $fn:literal, $ret:literal) => {
			gen_cmp(gen, concat!("Partial", $trait), $fn, target, other, $ret, concat!("self.0.", $fn, "(other)"))?;
			gen_cmp(gen, concat!("Partial", $trait), $fn, other, target, $ret, concat!("self.", $fn, "(&other.0)"))?;
		};
	}

	gen_cmp!("Eq",  "eq", "bool");
	gen_cmp!("Ord", "partial_cmp", "Option<core::cmp::Ordering>");
	Ok(())
}

fn gen_cmp<'a>(
	gen: &mut Generator,
	trait_name: &str,
	fn_name: &str,
	target: &'a str,
	other_type: &'a str,
	ret_type: &'a str,
	body: &'a str
) -> Result {
	Impl::from(
		Comparison { target, other_type, ret_type, body }
	).build(gen, trait_name, fn_name)
}
