extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{
    Span,
    TokenStream as TokenStream2,
};
use quote::{
    format_ident,
    quote,
};
use syn::{
    ConstParam,
    Data,
    DataStruct,
    DeriveInput,
    GenericParam,
    Index,
    TypeParam,
    parse_macro_input,
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
};

/// Creates a noise operation.
#[proc_macro]
pub fn noise_op(_item: TokenStream) -> TokenStream {
    println!("Yay");
    quote! {}.into()
}
