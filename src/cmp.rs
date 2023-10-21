// SPDX-License-Identifier: Apache-2.0

use virtue::prelude::*;

pub fn expand_eq(gen: &mut Generator, target: &str, other: &str) -> Result {
	expand_cmp(gen, "Eq", "eq", "bool", target, other, "self.0.eq(other)")?;
	expand_cmp(gen, "Eq", "eq", "bool", other, target, "self.eq(&other.0)")
}

pub fn expand_ord(gen: &mut Generator, target: &str, other: &str) -> Result {
	const ORDERING: &str = "Option<core::cmp::Ordering>";
	expand_cmp(gen, "Ord", "partial_cmp", ORDERING, target, other, "self.0.partial_cmp(other)")?;
	expand_cmp(gen, "Ord", "partial_cmp", ORDERING, other, target, "self.partial_cmp(&other.0)")
}

fn expand_cmp(
	gen: &mut Generator,
	name: &str,
	fn_name: &str,
	fn_ret: &str,
	target: &str,
	other: &str,
	comparison: &str
) -> Result {
	let name = format!("Partial{name}<{other}>");
	let mut gen = gen.impl_trait_for_other_type(name, target);
	gen.impl_outer_attr("automatically_derived")?;
	gen.generate_fn(fn_name)
	   .with_self_arg(FnSelfArg::RefSelf)
	   .with_arg("other", format!("&{other}"))
	   .with_return_type(fn_ret)
	   .body(|body| {
		   body.push_parsed(comparison)?;
		   Ok(())
	   })
}
