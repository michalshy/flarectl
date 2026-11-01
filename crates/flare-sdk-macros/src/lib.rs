use proc_macro::TokenStream;

// pass-through for now, so the examples compile
#[proc_macro_attribute]
pub fn job(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

// pass-through for now, so the examples compile
#[proc_macro_attribute]
pub fn step(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
