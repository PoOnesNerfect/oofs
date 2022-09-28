use super::{action::Action, write::write};
use quote::ToTokens;
use syn::{parse::Parse, ItemFn};

pub struct OofFn {
    pub actions: Vec<Action>,
    pub inner: ItemFn,
}

impl Parse for OofFn {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let item_fn: ItemFn = input.parse()?;
        let mut actions = Vec::new();

        for attr in &item_fn.attrs {
            if attr.path.is_ident("oofs") {
                let tokens = &attr.tokens;
                actions.push(syn::parse_quote!(#tokens));
            }
        }

        Ok(Self {
            actions,
            inner: item_fn,
        })
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
