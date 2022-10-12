use super::{props, Props};
use quote::ToTokens;
use syn::{parse::Parse, ItemFn};

pub struct OofFn {
    pub inner: ItemFn,
    pub props: Props,
}

impl OofFn {
    pub fn with_props(mut self, props: Props) -> Self {
        self.props.merge(props);
        self
    }
}

impl Parse for OofFn {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let item_fn: ItemFn = input.parse()?;

        Ok(Self {
            inner: item_fn,
            props: props(),
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

        let mut props = self.props.clone();

        for attr in attrs {
            if !props.merge_attr(attr) {
                attr.to_tokens(tokens);
            }
        }
        vis.to_tokens(tokens);
        sig.to_tokens(tokens);

        if props.skip() {
            block.to_tokens(tokens);
        } else {
            props.write(tokens).block(block);
        }
    }
}
