use crate::embedded_derive_macro::embedded_derive_macro_impl;
use proc_macro::TokenStream;

use crate::model_derive_macro::model_derive_macro_impl;
use crate::update_model_derive_macro::update_model_derive_macro_impl;

mod common;
mod embedded_derive_macro;
mod model_derive_macro;
mod update_model_derive_macro;

#[proc_macro_derive(Model, attributes(dojo))]
pub fn model_derive_macro(input: TokenStream) -> TokenStream {
    model_derive_macro_impl(input.into()).unwrap().into()
}

#[proc_macro_derive(EmbeddedModel)]
pub fn embedded_derive_macro(input: TokenStream) -> TokenStream {
    embedded_derive_macro_impl(input.into()).unwrap().into()
}

#[proc_macro_derive(UpdateModel, attributes(dojo))]
pub fn update_model_derive_macro(input: TokenStream) -> TokenStream {
    update_model_derive_macro_impl(input.into()).unwrap().into()
}
