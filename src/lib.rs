// SPDX-License-Identifier: Apache-2.0

use proc_macro::TokenStream;

#[proc_macro_derive(Primitive)]
pub fn primitive_derive(input: TokenStream) -> TokenStream {
	TokenStream::new()
}
