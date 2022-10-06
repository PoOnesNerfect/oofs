use super::write::write;
use quote::ToTokens;
use syn::{parse::Parse, ItemFn};

pub struct OofFn {
    pub inner: ItemFn,
}

impl Parse for OofFn {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let item_fn: ItemFn = input.parse()?;

        Ok(Self { inner: item_fn })
    }
}

impl ToTokens for OofFn {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ItemFn {
            attrs,
            vis,
            sig,
            block,
        } = &self.inner;

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        vis.to_tokens(tokens);
        sig.to_tokens(tokens);

        write(tokens).block(block);
    }
}
