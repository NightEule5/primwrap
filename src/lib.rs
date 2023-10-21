// SPDX-License-Identifier: Apache-2.0

#![feature(try_blocks)]

mod cmp;
mod ops;

use proc_macro::TokenStream;
use strum::IntoEnumIterator;
use virtue::parse::StructBody;
use virtue::prelude::*;
use crate::cmp::{expand_eq, expand_ord};
use crate::ops::{arithmetic_ops_for_type, Bit, Op};

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
	let ref target = gen.target_name().to_string();

	let result: Result = try {
		let inner_type = match inner_type.try_into() {
			Ok(it) => it,
			Err(e) => return e
		};

		match inner_type {
			Type::Int(ref inner) => {
				for op in arithmetic_ops_for_type(inner) {
					op.expand(&mut gen, target, inner)?;
				}

				for op in Bit::iter() {
					op.expand(&mut gen, target, inner)?;
				}
			}
			Type::Bool(ref inner) => Bit::Not.expand(&mut gen, target, inner)?
		}

		let (Type::Int(ref inner) | Type::Bool(ref inner)) = inner_type;
		expand_eq (&mut gen, target, inner)?;
		expand_ord(&mut gen, target, inner)?;
	};
	if let Err(error) = result {
		return error.into_token_stream()
	}

	gen.finish()
	   .unwrap_or_else(Error::into_token_stream)
}
