// SPDX-License-Identifier: Apache-2.0

#![feature(try_blocks)]

mod cmp;
mod ops;

use proc_macro::TokenStream;
use std::collections::HashSet;
use strum::{EnumIter, EnumString, IntoEnumIterator};
use virtue::parse::{Attribute, StructBody};
use virtue::prelude::*;
use crate::cmp::{expand_eq, expand_ord};
use crate::ops::{arithmetic_ops_for_type, Bit, bit_ops_for_bool, Op};

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
			_ => Err(Error::custom_at("unknown type", value.span()))
		}
	}
}

#[derive(EnumIter, EnumString, Eq, PartialEq, Hash)]
#[strum(ascii_case_insensitive)]
enum Group {
	Arithmetic,
	Bitwise,
	Formatting,
	Comparison
}

#[proc_macro_derive(Primitive, attributes(primwrap))]
pub fn primitive_derive(input: TokenStream) -> TokenStream {
	let parsed = match Parse::new(input) {
		Ok(parsed) => parsed,
		Err(error) => return error.into_token_stream()
	};

	let attributes = if let Parse::Struct { ref attributes, .. } |
							Parse::Enum   { ref attributes, .. } = parsed {
		attributes
	} else {
		unreachable!()
	};
	let groups = match parse_attributes(attributes) {
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
				if groups.contains(&Group::Arithmetic) {
					for op in arithmetic_ops_for_type(inner) {
						op.expand(&mut gen, target, inner)?;
					}
				}

				if groups.contains(&Group::Bitwise) {
					for op in Bit::iter() {
						op.expand(&mut gen, target, inner)?;
					}
				}
			}
			Type::Bool(ref inner) =>
				if groups.contains(&Group::Bitwise) {
					for op in bit_ops_for_bool() {
						op.expand(&mut gen, target, inner)?;
					}
				}
		}

		if groups.contains(&Group::Comparison) {
			let (Type::Int(ref inner) | Type::Bool(ref inner)) = inner_type;
			expand_eq (&mut gen, target, inner)?;
			expand_ord(&mut gen, target, inner)?;
		}
	};
	if let Err(error) = result {
		return error.into_token_stream()
	}

	gen.finish()
	   .unwrap_or_else(Error::into_token_stream)
}

fn parse_attributes(attributes: &Vec<Attribute>) -> Result<HashSet<Group>> {
	fn convert_error<T>(result: syn::Result<T>) -> Result<T> {
		result.map_err(|err| Error::custom_at(err.to_string(), err.span().unwrap()))
	}

	for Attribute { tokens, .. } in attributes.iter() {
		let stream = tokens.stream();
		let meta: syn::Meta = convert_error(syn::parse(stream))?;
		let list = convert_error(meta.require_list())?;
		if !list.path.is_ident("primwrap") { continue }

		let mut groups = HashSet::with_capacity(4);
		convert_error(list.parse_nested_meta(|meta| {
			let ident = meta.path.require_ident()?.to_string();
			let group = ident.parse().map_err(|_|
				meta.input.error(r#"expected "arithmetic", "bitwise", "formatting", or "comparison""#)
			)?;
			groups.insert(group);
			Ok(())
		}))?;

		if !groups.is_empty() {
			return Ok(groups)
		}
	}

	Ok(Group::iter().collect())
}
