mod setting;
mod settings;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn settings(attr: TokenStream, item: TokenStream) -> TokenStream {
    settings::expand(attr.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
