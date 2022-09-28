use super::OofMode;
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use syn::{parenthesized, parse::Parse, Ident};

pub enum Action {
    Tag(Tag),
}

impl Parse for Action {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut content;
        parenthesized!(content in input);

        let mut parse_remove = || {
            let ident = content.parse()?;
            parenthesized!(content in content);

            Ok(ident)
        };

        let mut ident: Ident = parse_remove()?;

        let mode = if ident == "debug" {
            ident = parse_remove()?;
            OofMode::Debug
        } else if ident == "test" {
            ident = parse_remove()?;
            OofMode::Test
        } else if ident == "disabled" {
            ident = parse_remove()?;
            OofMode::Disabled
        } else {
            OofMode::Always
        };

        if ident == "tag" {
            content
                .parse::<Tag>()
                .map(|t| t.with_mode(mode))
                .map(Self::Tag)
        } else {
            abort!(
                ident,
                "unexpected action `{}`", ident;
                help = "use one of `debug`, `release`, `tag`"
            );
        }
    }
}

pub struct Tag {
    mode: OofMode,
    name: Ident,
}

impl Tag {
    fn with_mode(mut self, mode: OofMode) -> Self {
        self.mode = mode;
        self
    }
}

impl Parse for Tag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            mode: OofMode::Always,
            name: input.parse()?,
        })
    }
}

impl ToTokens for Tag {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(quote! {});
    }
}
