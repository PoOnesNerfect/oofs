use proc_macro_error::proc_macro_error;
use quote::ToTokens;
use syn::parse_macro_input;

mod implementation;

#[proc_macro_error]
#[proc_macro_attribute]
pub fn oofs(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    if !args.is_empty() {
        return input;
    }

    let tokens = input.clone();
    let oofs = parse_macro_input!(tokens as implementation::Oofs);

    oofs.to_token_stream().into()
}
