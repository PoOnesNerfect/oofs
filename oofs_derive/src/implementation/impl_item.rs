use super::{props::props, Props};
use quote::ToTokens;
use syn::{parse::Parse, ImplItem, ImplItemMethod, ItemImpl, ReturnType, Signature, Type};

pub struct OofImpl {
    pub inner: ItemImpl,
    pub props: Props,
}

impl OofImpl {
    pub fn with_props(mut self, props: Props) -> Self {
        self.props.merge(props);
        self
    }
}

impl Parse for OofImpl {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let item_impl: ItemImpl = input.parse()?;

        Ok(Self {
            inner: item_impl,
            props: props(),
        })
    }
}

impl ToTokens for OofImpl {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ItemImpl {
            attrs,
            defaultness,
            unsafety,
            impl_token,
            generics,
            trait_,
            self_ty,
            brace_token,
            items,
        } = &self.inner;

        let mut impl_props = self.props.clone();

        for attr in attrs {
            if !impl_props.merge_attr(&attr) {
                attr.to_tokens(tokens);
            }
        }

        defaultness.to_tokens(tokens);
        unsafety.to_tokens(tokens);
        impl_token.to_tokens(tokens);
        generics.to_tokens(tokens);
        if let Some((bang_token, path, for_token)) = trait_ {
            bang_token.to_tokens(tokens);
            path.to_tokens(tokens);
            for_token.to_tokens(tokens);
        }
        self_ty.to_tokens(tokens);
        generics.where_clause.to_tokens(tokens);

        brace_token.surround(tokens, |braces| {
            for item in items {
                if let ImplItem::Method(method) = item {
                    let ImplItemMethod {
                        attrs,
                        vis,
                        defaultness,
                        sig,
                        block,
                    } = method;

                    let mut fn_props = impl_props.clone();

                    let mut attr_exists = false;
                    for attr in attrs {
                        if fn_props.merge_attr(&attr) {
                            attr_exists = true;
                        } else {
                            attr.to_tokens(braces);
                        }
                    }

                    vis.to_tokens(braces);
                    defaultness.to_tokens(braces);
                    sig.to_tokens(braces);

                    if fn_props.skip || !(attr_exists || returns_result(sig)) {
                        block.to_tokens(braces);
                    } else {
                        fn_props.write(braces).block(block);
                    }
                } else {
                    item.to_tokens(braces);
                }
            }
        });
    }
}

fn returns_result(sig: &Signature) -> bool {
    if let ReturnType::Type(_, ty) = &sig.output {
        if let Type::Path(path) = ty.as_ref() {
            return path
                .path
                .segments
                .last()
                .map(|s| s.ident == "Result")
                .unwrap_or(false);
        }
    }

    false
}
