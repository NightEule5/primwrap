// SPDX-License-Identifier: Apache-2.0

mod accum;
mod cmp;
mod fmt;
mod ops;
mod util;

use proc_macro::TokenStream;
use strum::{EnumIter, EnumString, IntoEnumIterator};
use virtue::parse::{Attribute, StructBody};
use virtue::prelude::*;
use crate::accum::generate_accum;
use crate::cmp::generate_cmp;
use crate::fmt::generate_fmt;
use crate::ops::{Arithmetic, Bit, Op};

#[derive(EnumIter, EnumString, Eq, PartialEq)]
#[strum(ascii_case_insensitive)]
enum Group {
	Arithmetic,
	Bitwise,
	Formatting,
	Comparison,
	Accumulation
}

/// Derives arithmetic, bitwise, comparison, and formatting traits on a primitive
/// wrapper struct, exposing its inner type. Integer, float, and boolean types are
/// supported.
///
/// The implemented traits can be selected with a `#[primwrap(...)]` attribute:
/// - `arithmetic` enables `Add`, `Sub`, `Mul`, `Div`, `Rem`, and `Neg`
/// - `bitwise` enables `Not`, `BitAnd`, `BitOr`, `BitXor`, `Shl`, and `Shr`
/// - `formatting` enables `Debug`, `Display`, `Binary`, `Octal`, `LowerExp`,
///   `LowerHex`, `UpperExp`, and `UpperHex`
/// - `comparison` enables `PartialEq`/`PartialOrd` with the inner type
/// - `accumulation` enables `Sum` and `Product`
#[proc_macro_derive(Primitive, attributes(primwrap))]
pub fn primitive_derive(input: TokenStream) -> TokenStream {
	let expand = || {
		let parsed = Parse::new(input)?;
		let groups = if let Parse::Struct { ref attributes, .. } = parsed {
			parse_attributes(attributes)?
		} else {
			Vec::default()
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
			return Err(Error::custom("expected tuple struct"))
		};

		let [field] = &fields[..] else {
			return Err(Error::custom("expected tuple struct with one field"))
		};

		let [TokenTree::Ident(inner_type)] = &field.r#type[..] else {
			return Err(Error::custom("unknown type"))
		};
		let ref target = gen.target_name().to_string();
		let ref inner = inner_type.to_string();

		let has_arith = groups.contains(&Group::Arithmetic);
		for group in groups {
			match group {
				Group::Arithmetic => Arithmetic::generate_all(&mut gen, target, inner)?,
				Group::Bitwise => Bit::generate_all(&mut gen, target, inner)?,
				Group::Formatting => generate_fmt(&mut gen, target, inner)?,
				Group::Comparison => generate_cmp(&mut gen, target, inner)?,
				Group::Accumulation => generate_accum(&mut gen, has_arith, target, inner)?,
			}
		}

		gen.finish()
	};

	expand().unwrap_or_else(Error::into_token_stream)
}

fn parse_attributes(attributes: &Vec<Attribute>) -> Result<Vec<Group>> {
	fn convert_error<T>(result: syn::Result<T>) -> Result<T> {
		result.map_err(|err| Error::custom_at(err.to_string(), err.span().unwrap()))
	}

	for Attribute { tokens, .. } in attributes.iter() {
		let stream = tokens.stream();
		let meta: syn::Meta = convert_error(syn::parse(stream))?;
		let list = convert_error(meta.require_list())?;
		if !list.path.is_ident("primwrap") { continue }

		let mut groups = Vec::with_capacity(4);
		convert_error(list.parse_nested_meta(|meta| {
			let ident = meta.path.require_ident()?.to_string();
			let group = ident.parse().map_err(|_|
				meta.input.error(r#"expected "arithmetic", "bitwise", "formatting", or "comparison""#)
			)?;
			groups.push(group);
			Ok(())
		}))?;

		if !groups.is_empty() {
			return Ok(groups)
		}
	}

	Ok(Group::iter().collect())
}
