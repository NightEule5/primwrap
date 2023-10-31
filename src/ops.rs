// SPDX-License-Identifier: Apache-2.0

pub mod accum;
pub mod arith;
pub mod bit;
pub mod cmp;
pub mod fmt;

use std::borrow::Cow;
use crate::gen::Sink;
pub use arith::Arithmetic;
pub use accum::Accumulation;
pub use bit::Bit;
pub use cmp::Comparison;
pub use fmt::Formatting;

macro_rules! filter_assign {
	(with_assign $($tokens:tt)+) => {
		$($tokens)+
	};
	($($tokens:tt)+) => {
		unreachable!()
	}
}

use filter_assign;

macro_rules! expand_fns {
	// Special case for formatting trait group, where all functions have identical
	// signatures.
	($self:ident $op_name1:ident = $fn_name:literal, $($op_name:ident),+) => {
		$fn_name
	};
	($self:ident $($op_name:ident = $fn_name:literal),+) => {
		match $self {
			$(Self::$op_name => $fn_name),+
		}
	};
}

use expand_fns;

macro_rules! op_enum {
	(enum $name:ident in $op_path:ident { $($op_name:ident$( = $fn_name:literal)? $($op_type:ident)?),+ }) => {
		op_enum! {
			impl $name {
				$($op_name$( = $fn_name)? $($op_type)? @ core::$op_path::$op_name),+
			}
		}
	};
	(enum $name:ident { $($op_name:ident$( = $fn_name:literal)? $($op_type:ident)?),+ }) => {
		op_enum! {
			impl $name {
				$($op_name$( = $fn_name)? $($op_type)? @ $op_name),+
			}
		}
	};
	(impl $name:ident { $($op_name:ident$( = $fn_name:literal)? $($op_type:ident)? @ $($op_path:ident)::+),+ }) => {
		use super::AssignOpData;

		#[derive(Clone, Copy)]
		pub enum $name {
			$($op_name),+
		}

		impl $name {
			#[allow(dead_code)]
			const VARS: &'static [Self] = &[$(Self::$op_name),+];
		}

		impl super::OpData for $name {
			fn trait_name(&self) -> &'static str {
				match self {
					$(Self::$op_name => stringify!($($op_path)::+)),+
				}
			}

			fn fn_name(&self) -> &'static str {
				super::expand_fns!(self $($op_name$( = $fn_name)?),+)
			}

			fn assign(&self) -> &'static AssignOpData {
				match self {
					$(
					Self::$op_name => super::filter_assign!($($op_type)? &AssignOpData {
						trait_name: concat!(stringify!($($op_path)::+), "Assign"),
						fn_name: concat!($($fn_name)?, "_assign")
					})
					),+
				}
			}
		}
	};
}

use op_enum;

pub struct Binary<'a> {
	pub target: &'a str,
	pub output: &'a str,
	pub lhs: &'a str,
	pub lhs_mut: bool,
	pub rhs_bind: Cow<'a, str>,
	pub rhs_type: &'a str,
	pub base_ret: Option<String>,
}

pub struct Comparing<'a> {
	pub target: &'a str,
	pub other_bind: Cow<'a, str>,
	pub other_type: &'a str,
	pub body: &'a str,
}

pub struct Accumulating<'a> {
	pub target: &'a str,
	pub element_type: &'a str,
	pub body: String,
}

impl<'a> Binary<'a> {
	fn permutations(target: &'a str, inner: &'a str) -> [Self; 3] {
		[
			Binary::self_self(target),
			Binary::self_inner(target, inner),
			Binary::inner_self(target, inner)
		]
	}

	fn inner_permutations(target: &'a str, inner: &'a str, other: &'a str) -> [Self; 2] {
		[
			Binary::self_inner(target, other),
			Binary::other_self(target, inner, other)
		]
	}

	fn self_self(target: &'a str) -> Self {
		Self {
			target,
			output: "Self",
			lhs: "self.0",
			lhs_mut: true,
			rhs_bind: "Self(rhs)".into(),
			rhs_type: "Self",
			base_ret: None,
		}
	}

	fn self_inner(target: &'a str, inner: &'a str) -> Self {
		Self {
			target,
			output: target,
			lhs: "self.0",
			lhs_mut: true,
			rhs_bind: "rhs".into(),
			rhs_type: inner,
			base_ret: None,
		}
	}

	fn inner_self(target: &'a str, inner: &'a str) -> Self {
		Self {
			target: inner,
			output: target,
			lhs: "self",
			lhs_mut: false,
			rhs_bind: format!("{target}(rhs)").into(),
			rhs_type: target,
			base_ret: Some(format!("{target}(self)")),
		}
	}

	fn other_self(target: &'a str, inner: &'a str, other: &'a str) -> Self {
		Self {
			target: other,
			output: target,
			lhs: "self",
			lhs_mut: false,
			rhs_bind: format!("{target}(rhs)").into(),
			rhs_type: target,
			base_ret: Some(format!("{target}(self as {inner})"))
		}
	}
}

impl<'a> Accumulating<'a> {
	fn self_self(target: &'a str, fn_name: &str) -> Self {
		Self {
			target,
			element_type: "Self",
			body: format!("Self(iter.{fn_name}())"),
		}
	}

	fn self_inner(target: &'a str, inner: &'a str, fn_name: &str) -> Self {
		Self {
			target,
			element_type: inner,
			body: format!("Self(iter.{fn_name}())")
		}
	}

	fn inner_self(target: &'a str, inner: &'a str, fn_name: &str) -> Self {
		Self {
			target: inner,
			element_type: target,
			body: format!("iter.map(|it| it.0).{fn_name}()")
		}
	}
}

pub struct AssignOpData {
	trait_name: &'static str,
	fn_name: &'static str
}

pub trait OpData {
	fn trait_name(&self) -> &'static str;
	fn fn_name(&self) -> &'static str;
	fn assign(&self) -> &'static AssignOpData;
}

impl OpData for AssignOpData {
	fn trait_name(&self) -> &'static str { self.trait_name }
	fn fn_name(&self) -> &'static str { self.fn_name }
	fn assign(&self) -> &'static AssignOpData { unimplemented!() }
}

pub trait Op: OpData + Sized {
	fn supported(type_name: &str) -> &[Self];

	fn generate_all<'a>(sink: &mut Sink<'a>, target: &'a str, inner: &'a str) where Self: 'a {
		for op in Self::supported(inner) {
			op.generate(sink, target, inner);
		}
	}

	fn generate<'a>(&'a self, sink: &mut Sink<'a>, target: &'a str, inner: &'a str);
}
