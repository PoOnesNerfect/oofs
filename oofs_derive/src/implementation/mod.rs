use quote::ToTokens;
use syn::{parse::Parse, Attribute, Token};

mod context;
mod fn_item;
mod impl_item;
mod props;
mod write;

pub use props::*;

// TODO FEATURES:
// - pre-check. ex) #[oof(pre(!list.len().is_empty()))], ex) #[oof(pre(!list.len().is_empty(), message = "custom message"))]
// - post-check. ex) #[oof(post(!list.len().is_empty()))]
// - invar: pre + post. ex) #[oof(invar(!list.len().is_empty()))]

use self::{fn_item::OofFn, impl_item::OofImpl};

pub enum Oofs {
    Impl(impl_item::OofImpl),
    Fn(fn_item::OofFn),
}

impl Oofs {
    pub fn with_args(self, args: PropArgs) -> Self {
        use Oofs::*;

        let props = args.into();

        match self {
            Impl(t) => Impl(t.with_props(props)),
            Fn(t) => Fn(t.with_props(props)),
        }
    }
}

impl Parse for Oofs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = Attribute::parse_outer(input)?;

        let mut lookahead = input.lookahead1();

        if lookahead.peek(Token![unsafe]) {
            let ahead = input.fork();
            ahead.parse::<Token![unsafe]>()?;
            lookahead = ahead.lookahead1();
        }

        if lookahead.peek(Token![impl]) || lookahead.peek(Token![default]) {
            let mut item: OofImpl = input.parse()?;
            item.inner.attrs = attrs;
            Ok(Self::Impl(item))
        } else if lookahead.peek(Token![fn])
            || lookahead.peek(Token![pub])
            || lookahead.peek(Token![async])
            || lookahead.peek(Token![const])
        {
            let mut item: OofFn = input.parse()?;
            item.inner.attrs = attrs;
            Ok(Self::Fn(item))
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for Oofs {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Impl(oof_impl) => oof_impl.to_tokens(tokens),
            Self::Fn(oof_fn) => oof_fn.to_tokens(tokens),
        }
    }
}
