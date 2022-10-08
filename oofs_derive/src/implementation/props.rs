use super::{context::Context, write::Writer};
use proc_macro_error::abort;
use std::ops::{Deref, DerefMut};
use syn::{
    parenthesized, parse::Parse, parse_quote, punctuated::Punctuated, Attribute, Expr, Ident,
    Token, Type,
};

pub fn props() -> Properties {
    Properties::default()
}

#[derive(Clone)]
pub struct Properties {
    pub args: PropArgs,
}

impl From<PropArgs> for Properties {
    fn from(args: PropArgs) -> Self {
        Self { args }
    }
}

impl Deref for Properties {
    type Target = PropArgs;

    fn deref(&self) -> &Self::Target {
        &self.args
    }
}

impl DerefMut for Properties {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.args
    }
}

impl Default for Properties {
    fn default() -> Self {
        Self {
            args: Default::default(),
        }
    }
}

impl Properties {
    pub fn write<'a>(&self, tokens: &'a mut proc_macro2::TokenStream) -> Writer<'a> {
        Writer::new(tokens, self.clone())
    }

    pub fn context<'a>(&self, tokens: &'a mut proc_macro2::TokenStream) -> Context<'a> {
        Context::new(tokens, self.clone())
    }

    pub fn merge(&mut self, other: Properties) {
        self.args.merge(other.args);
    }

    pub fn merge_attr(&mut self, attr: &Attribute) -> bool {
        let Attribute { path, tokens, .. } = attr;
        if !path.is_ident("oofs") {
            return false;
        }

        if !tokens.is_empty() {
            self.merge(parse_quote!(#tokens));
        }

        true
    }
}

impl Parse for Properties {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self::default());
        }

        let content;
        parenthesized!(content in input);

        let args: PropArgs = content.parse()?;

        Ok(Self { args })
    }
}

#[derive(Clone)]
pub struct PropArgs {
    pub closures: bool,
    pub async_blocks: bool,
    pub skip: bool,
    pub tags: Vec<Type>,
    pub attach: Vec<Expr>,
    pub attach_lazy: Vec<Expr>,
}

impl PropArgs {
    pub fn merge(&mut self, other: PropArgs) {
        self.closures |= other.closures;
        self.async_blocks |= other.async_blocks;
        self.skip |= other.skip;
        self.tags.extend(other.tags);
        self.attach.extend(other.attach);
        self.attach_lazy.extend(other.attach_lazy);
    }
}

impl Default for PropArgs {
    fn default() -> Self {
        Self {
            closures: false,
            async_blocks: false,
            skip: false,
            tags: Vec::new(),
            attach: Vec::new(),
            attach_lazy: Vec::new(),
        }
    }
}

impl Parse for PropArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let parsed: Punctuated<PropArg, Token!(,)> = Punctuated::parse_terminated(input)?;

        let mut args = PropArgs::default();

        for arg in parsed {
            use PropArg::*;
            match arg {
                Closures => args.closures = true,
                AsyncBlocks => args.async_blocks = true,
                Skip => args.skip = true,
                Tags(tags) => args.tags = tags,
                Attach(attach) => args.attach = attach,
                AttachLazy(attach_lazy) => args.attach_lazy = attach_lazy,
            }
        }

        Ok(args)
    }
}

#[derive(Clone)]
pub enum PropArg {
    Closures,
    AsyncBlocks,
    Skip,
    Tags(Vec<Type>),
    Attach(Vec<Expr>),
    AttachLazy(Vec<Expr>),
}

impl Parse for PropArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;

        let arg = if ident == "closures" {
            Self::Closures
        } else if ident == "async_blocks" {
            Self::AsyncBlocks
        } else if ident == "skip" {
            Self::Skip
        } else if ident == "tags" {
            let content;
            parenthesized!(content in input);

            let tags: Punctuated<Type, Token!(,)> = Punctuated::parse_terminated(&content)?;

            Self::Tags(tags.into_iter().collect())
        } else if ident == "attach" {
            let content;
            parenthesized!(content in input);

            let attach: Punctuated<Expr, Token!(,)> = Punctuated::parse_terminated(&content)?;

            Self::Attach(attach.into_iter().collect())
        } else if ident == "attach_lazy" {
            let content;
            parenthesized!(content in input);

            let attach_lazy: Punctuated<Expr, Token!(,)> = Punctuated::parse_terminated(&content)?;

            Self::AttachLazy(attach_lazy.into_iter().collect())
        } else {
            abort!(ident, "Expected one of `closures`, `async_blocks`, `tags`");
        };

        Ok(arg)
    }
}
