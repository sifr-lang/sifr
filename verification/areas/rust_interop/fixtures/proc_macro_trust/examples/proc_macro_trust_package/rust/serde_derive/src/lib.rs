extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro_derive(SifrGenerated)]
pub fn sifr_generated(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
