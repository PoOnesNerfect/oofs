use super::{
    util::{OpLevel, OpWrapper},
    write::write,
};
use quote::ToTokens;
use syn::{
    parse::Parse, Expr, ImplItem, ImplItemMethod, ItemImpl, PathArguments, ReturnType, Type,
};

pub struct OofImpl {
    pub pre: Vec<OpWrapper<Expr>>,
    pub post: Vec<OpWrapper<Expr>>,
    pub tags: Vec<OpWrapper<Type>>,
    pub display_owned: OpLevel,
    pub inner: ItemImpl,
}

impl Parse for OofImpl {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let item_impl: ItemImpl = input.parse()?;

        // for attr in &item_impl.attrs {
        //     if attr.path.is_ident("oofs") {
        //         let tokens = &attr.tokens;
        //         actions.push(syn::parse_quote!(#tokens));
        //     }
        // }

        Ok(Self {
            pre: Vec::new(),
            post: Vec::new(),
            tags: Vec::new(),
            display_owned: OpLevel::Debug,
            inner: item_impl,
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

        for attr in attrs {
            attr.to_tokens(tokens);
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
                if !return_type_is_result(item) {
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

                    for attr in attrs {
                        attr.to_tokens(braces);
                    }
                    vis.to_tokens(braces);
                    defaultness.to_tokens(braces);
                    sig.to_tokens(braces);

                    write(braces).block(block);
                }
            }
        });
    }
}

fn return_type_is_result(item: &ImplItem) -> bool {
    if let ImplItem::Method(method) = item {
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
