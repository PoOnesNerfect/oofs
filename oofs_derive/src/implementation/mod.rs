use proc_macro2::Span;
use proc_macro_error::abort;
use quote::ToTokens;
use syn::{parse::Parse, Attribute, ImplItem, ItemFn, Token};

use self::{fn_item::OofFn, impl_item::OofImpl};

const OOF_METHODS: &[&str] = &[
    "with_oof_builder",
    "oof",
    "tag",
    "tag_if",
    "display_owned",
    "add_context",
];

mod action;
mod call_chain;
mod fn_item;
mod impl_item;
mod oof;
mod util;
mod write;

// FEATURES:
// - `impl Struct`, and `impl Trait for struct`
// - `fn any_function`
// - return cases:
//   - question marks: `some_method()?`
//   - return statements: `return some_method();` or `return Err(err);`
//   - last line: `some_method()` or `Err(err)`
// - pre-check. ex) #[oof(pre(!list.len().is_empty()))], ex) #[oof(pre(!list.len().is_empty(), message = "custom message"))]
// - post-check. ex) #[oof(post(!list.len().is_empty()))]
// - invar: pre + post. ex) #[oof(invar(!list.len().is_empty()))]
// - debug mode: ex) #[oof(debug(pre(...)))], #[oof(debug(tag(...)))]]
// - release mode: ex) #[oof(release(pre(...)))], #[oof(release(tag(...)))]
// - tag:
//    - all: #[oof(tag(MyTag))]
//    - by fn invocations: #[oof(MyTag2: [fn_name_1, fn_name_2])]
// - skip for function in impl. ex) #[oof(skip)]
// - custom display methods
//    - skip: #[oof(display(skip))]
//    - custom display methods:
//      - #[oof(display(serde_json::to_string))]
//      - #[oof(display(serde_json::to_string: [var1, var2]))]
// - display_owned:
//    - by default, only display referenced values
//    - allow for release with #[oof(display_owned(release))]
//    - allow for the whole package with feature `display_owned_release`.
//    - disable for debug with #[oof(display_owned(disabled))].
//    - disable for whole package with `display_owned_disabled`.
//    - allow for individual method invocation with `.display_owned()`.
// - different cases:
//   - future is initialized and .awaited in a different line.
//   - passed parameter is a closure or an async closure.
//   - function/method is chained with other methods that also return a result.

pub enum Oofs {
    Impl(impl_item::OofImpl),
    Fn(fn_item::OofFn),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum OofMode {
    Always,
    Disabled,
    Debug,
    Test,
}

impl Parse for Oofs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = Attribute::parse_outer(input)?;

        let mut lookahead = input.lookahead1();
        let ahead = input.fork();

        if lookahead.peek(Token![unsafe]) {
            ahead.parse::<Token![unsafe]>()?;
            lookahead = ahead.lookahead1();
        }

        if lookahead.peek(Token![impl]) || lookahead.peek(Token![default]) {
            let mut item: OofImpl = input.parse()?;
            item.inner.attrs = attrs;
            return Ok(Self::Impl(item));
        }

        if lookahead.peek(Token![pub]) {
            ahead.parse::<Token![pub]>()?;
            lookahead = ahead.lookahead1();
        }

        if lookahead.peek(Token![const]) {
            ahead.parse::<Token![const]>()?;
            lookahead = ahead.lookahead1();
        }

        if lookahead.peek(Token![fn]) || lookahead.peek(Token![async]) {
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
