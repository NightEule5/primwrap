// SPDX-License-Identifier: Apache-2.0

#![feature(try_blocks)]

mod ops;

use proc_macro::TokenStream;
use virtue::parse::StructBody;
use virtue::prelude::*;
use crate::ops::{arithmetic_ops_for_type, Op};

enum Type {
	Int(String),
	Bool(String)
}

impl TryFrom<&Ident> for Type {
	type Error = TokenStream;
	fn try_from(value: &Ident) -> std::result::Result<Self, TokenStream> {
		let name = value.to_string();
		match &*name {
			"i8"    | "u8"   |
			"i16"   | "u16"  |
			"i32"   | "u32"  |
			"i64"   | "u64"  |
			"i128"  | "u128" |
			"isize" | "usize" => Ok(Self::Int(name)),
			"bool" => Ok(Self::Bool(name)),
			_ => Err(Error::custom("unknown type").throw_with_span(value.span()))
		}
	}
}

#[proc_macro_derive(Primitive)]
pub fn primitive_derive(input: TokenStream) -> TokenStream {
	let parsed = match Parse::new(input) {
		Ok(parsed) => parsed,
		Err(error) => return error.into_token_stream()
	};
	let (
		mut gen,
		_,
		Body::Struct(
			StructBody {
				fields: Some(Fields::Tuple(fields))
			}
		)
	) = parsed.into_generator() else {
		return Error::custom("expected tuple struct").into_token_stream()
	};

	let [field] = &fields[..] else {
		return Error::custom("expected tuple struct with one field").into_token_stream()
	};

	let [TokenTree::Ident(inner_type)] = &field.r#type[..] else {
		return Error::custom("unknown type").into_token_stream()
	};

	let target = gen.target_name().to_string();

	let result: Result = try {
		match inner_type.try_into() {
			Ok(Type::Int(ref inner)) => {
				for op in arithmetic_ops_for_type(inner) {
					op.expand(&mut gen, &target, inner)?;
				}

				{
					let mut peq = gen.impl_for(format!("PartialEq<{inner}>"));
					peq.impl_outer_attr("automatically_derived")?;
					peq.generate_fn("eq")
					   .with_self_arg(FnSelfArg::RefSelf)
					   .with_arg("other", format!("&{inner}"))
					   .with_return_type("bool")
					   .body(|body| {
						   body.push_parsed("self.0.eq(other)")?;
						   Ok(())
					   })?;
				}
				{
					let mut peq = gen.impl_trait_for_other_type(format!("PartialEq<{target}>"), inner.clone());
					peq.impl_outer_attr("automatically_derived")?;
					peq.generate_fn("eq")
					   .with_self_arg(FnSelfArg::RefSelf)
					   .with_arg("other", format!("&{target}"))
					   .with_return_type("bool")
					   .body(|body| {
						   body.push_parsed("self.eq(&other.0)")?;
						   Ok(())
					   })?;
				}
				{
					let mut pord = gen.impl_for(format!("PartialOrd<{inner}>"));
					pord.impl_outer_attr("automatically_derived")?;
					pord.generate_fn("partial_cmp")
						.with_self_arg(FnSelfArg::RefSelf)
						.with_arg("other", format!("&{inner}"))
						.with_return_type("Option<core::cmp::Ordering>")
						.body(|body| {
							body.push_parsed("self.0.partial_cmp(other)")?;
							Ok(())
						})?;
				}
				{
					let mut pord = gen.impl_trait_for_other_type(format!("PartialOrd<{target}>"), inner.clone());
					pord.impl_outer_attr("automatically_derived")?;
					pord.generate_fn("partial_cmp")
						.with_self_arg(FnSelfArg::RefSelf)
						.with_arg("other", format!("&{target}"))
						.with_return_type("Option<core::cmp::Ordering>")
						.body(|body| {
							body.push_parsed("self.partial_cmp(&other.0)")?;
							Ok(())
						})?;
				}
			}
			Ok(Type::Bool(ref inner)) => {

			}
			Err(error) => return error
		}

	};
	if let Err(error) = result {
		return error.into_token_stream()
	}

	gen.finish()
	   .unwrap_or_else(Error::into_token_stream)
}
