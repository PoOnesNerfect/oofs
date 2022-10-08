use super::{props::props, Properties};
use quote::ToTokens;
use syn::{parse::Parse, ImplItem, ImplItemMethod, ItemImpl, ReturnType, Type};

pub struct OofImpl {
    pub inner: ItemImpl,
    pub props: Properties,
}

impl OofImpl {
    pub fn with_props(mut self, props: Properties) -> Self {
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
                if !should_impl_oof(item) {
                    item.to_tokens(braces);
                    continue;
                }

                if let ImplItem::Method(method) = item {
                    let ImplItemMethod {
                        attrs,
                        vis,
                        defaultness,
                        sig,
                        block,
                    } = method;

                    let mut fn_props = impl_props.clone();
                    for attr in attrs {
                        if !fn_props.merge_attr(&attr) {
                            attr.to_tokens(braces);
                        }
                    }

                    vis.to_tokens(braces);
                    defaultness.to_tokens(braces);
                    sig.to_tokens(braces);

                    fn_props.write(braces).block(block);
                }
            }
        });
    }
}

fn should_impl_oof(item: &ImplItem) -> bool {
    if let ImplItem::Method(method) = item {
        for attr in &method.attrs {
            let mut props = props();
            let oofs_merged = props.merge_attr(attr);

            if props.skip {
                return false;
            }

            if oofs_merged {
                return true;
            }
        }

        if let ReturnType::Type(_, ty) = &method.sig.output {
            if let Type::Path(path) = ty.as_ref() {
                return path
                    .path
                    .segments
                    .last()
                    .map(|s| s.ident == "Result")
                    .unwrap_or(false);
            }
        }
    }

    false
}
