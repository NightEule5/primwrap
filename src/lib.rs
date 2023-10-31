// SPDX-License-Identifier: Apache-2.0

mod gen;
mod ops;

use proc_macro::TokenStream;
use std::collections::HashSet;
use strum::EnumString;
use virtue::parse::{Attribute, AttributeLocation, StructBody};
use virtue::prelude::*;
use virtue::utils::{parse_tagged_attribute, ParsedAttribute};
use ops::*;

#[derive(EnumString, Eq, PartialEq, Hash)]
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
		let (
			gen,
			attributes,
			Body::Struct(
				StructBody {
					fields: Some(Fields::Tuple(fields))
				}
			)
		) = Parse::new(input)?.into_generator() else {
			return Err(Error::custom("expected tuple struct"))
		};
		let groups = parse_attributes(attributes)?;
		let [field] = &fields[..] else {
			return Err(Error::custom("expected tuple struct with one field"))
		};
		let [TokenTree::Ident(inner_type)] = &field.r#type[..] else {
			return Err(Error::custom("unknown type"))
		};
		let ref target = gen.target_name().to_string();
		let ref inner = inner_type.to_string();
		let mut impl_sink = gen.into();

		for group in &groups {
			match group {
				Group::Arithmetic => Arithmetic::generate_all(&mut impl_sink, target, inner),
				Group::Bitwise    => Bit       ::generate_all(&mut impl_sink, target, inner),
				Group::Formatting => Formatting::generate_all(&mut impl_sink, target, inner),
				Group::Comparison => Comparison::generate_all(&mut impl_sink, target, inner),
				Group::Accumulation
					if groups.contains(&Group::Arithmetic) =>
					Accumulation::generate_all(&mut impl_sink, target, inner),
				_ => { }
			}
		}

		impl_sink.finish()
	};

	expand().unwrap_or_else(Error::into_token_stream)
}

fn parse_attributes(attributes: Vec<Attribute>) -> Result<HashSet<Group>> {
	let attrs = attributes.iter().filter_map(|Attribute { location, tokens, .. }|
		matches!(location, AttributeLocation::Container).then_some(tokens)
	).collect::<Vec<_>>();

	if attrs.is_empty() {
		return Ok([
			Group::Arithmetic,
			Group::Bitwise,
			Group::Formatting,
			Group::Comparison,
			Group::Accumulation,
		].into())
	}

	let mut groups = HashSet::with_capacity(5);
	for tokens in attrs {
		let Some(parsed) = parse_tagged_attribute(tokens, "primwrap")? else { continue };
		for group in parsed {
			match group {
				ParsedAttribute::Tag(group) => {
					groups.insert(
						group.to_string()
							 .parse()
							 .map_err(|_|
								 Error::custom_at(
									 r#"expected "arithmetic", "bitwise", "formatting", or "comparison""#,
									 group.span()
								 )
							 )?
					);
				}
				ParsedAttribute::Property(_, val) =>
					return Err(Error::custom_at("expected identifier", val.span())),
				_ => return Err(Error::custom("expected identifier"))
			}
		}
	}
	Ok(groups)
}
