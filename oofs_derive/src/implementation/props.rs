use super::{context::Context, write::Writer};
use proc_macro_error::abort;
use std::ops::{Deref, DerefMut};
use syn::{
    parenthesized, parse::Parse, parse_quote, punctuated::Punctuated, token::Paren, Attribute,
    Expr, Ident, LitBool, Token, Type,
};

pub fn props() -> Props {
    Props::default()
}

#[derive(Clone)]
pub struct Props {
    pub args: PropArgs,
}

impl From<PropArgs> for Props {
    fn from(args: PropArgs) -> Self {
        Self { args }
    }
}

impl Deref for Props {
    type Target = PropArgs;

    fn deref(&self) -> &Self::Target {
        &self.args
    }
}

impl DerefMut for Props {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.args
    }
}

impl Default for Props {
    fn default() -> Self {
        Self {
            args: Default::default(),
        }
    }
}

impl Props {
    pub fn write<'a>(&'a self, tokens: &'a mut proc_macro2::TokenStream) -> Writer<'a> {
        Writer::new(tokens, self)
    }

    pub fn context<'a>(&'a self, tokens: &'a mut proc_macro2::TokenStream) -> Context<'a> {
        Context::new(tokens, self)
    }

    pub fn merge(&mut self, other: Props) {
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

impl Parse for Props {
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
    pub tag: Vec<Type>,
    pub attach: Vec<Expr>,
    pub attach_lazy: Vec<Expr>,
    pub skip_debug: Vec<Expr>,
}

impl PropArgs {
    pub fn merge(&mut self, other: PropArgs) {
        self.closures |= other.closures;
        self.async_blocks |= other.async_blocks;
        self.skip |= other.skip;
        self.tag.extend(other.tag);
        self.attach.extend(other.attach);
        self.attach_lazy.extend(other.attach_lazy);
        self.skip_debug.extend(other.skip_debug);
    }
}

impl Default for PropArgs {
    fn default() -> Self {
        Self {
            closures: false,
            async_blocks: false,
            skip: false,
            tag: Vec::new(),
            attach: Vec::new(),
            attach_lazy: Vec::new(),
            skip_debug: Vec::new(),
        }
    }
}

impl Parse for PropArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let parsed: Punctuated<Arg, Token!(,)> = Punctuated::parse_terminated(input)?;

        let mut args = PropArgs::default();

        for arg in parsed {
            use Arg::*;
            match arg {
                Closures(b) => args.closures = b,
                AsyncBlocks(b) => args.async_blocks = b,
                Skip(b) => args.skip = b,
                Tag(tag) => args.tag = tag,
                Attach(attach) => args.attach = attach,
                AttachLazy(attach_lazy) => args.attach_lazy = attach_lazy,
                SkipDebug(skip_debug) => args.skip_debug = skip_debug,
            }
        }

        Ok(args)
    }
}

#[derive(Clone)]
enum Arg {
    Closures(bool),
    AsyncBlocks(bool),
    Skip(bool),
    Tag(Vec<Type>),
    Attach(Vec<Expr>),
    AttachLazy(Vec<Expr>),
    SkipDebug(Vec<Expr>),
}

impl Parse for Arg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;

        let arg = if ident == "closures" {
            Self::Closures(extract_bool(input)?)
        } else if ident == "async_blocks" {
            Self::AsyncBlocks(extract_bool(input)?)
        } else if ident == "skip" {
            Self::Skip(extract_bool(input)?)
        } else if ident == "tag" {
            Self::Tag(extract_vec(input)?)
        } else if ident == "attach" {
            Self::Attach(extract_vec(input)?)
        } else if ident == "attach_lazy" {
            Self::AttachLazy(extract_vec(input)?)
        } else if ident == "skip_debug" {
            Self::SkipDebug(extract_vec(input)?)
        } else {
            abort!(ident, "Expected one of `closures`, `async_blocks`, `skip`, `tag`, `attach`, `attach_lazy`, `skip_debug`");
        };

        Ok(arg)
    }
}

fn extract_bool(input: syn::parse::ParseStream) -> syn::Result<bool> {
    let mut b = true;

    if input.peek(Paren) {
        let content;
        parenthesized!(content in input);

        let token: LitBool = content.parse()?;
        b = token.value();
    }

    Ok(b)
}

fn extract_vec<T: Parse>(input: syn::parse::ParseStream) -> syn::Result<Vec<T>> {
    let content;
    parenthesized!(content in input);

    let elems: Punctuated<T, Token!(,)> = Punctuated::parse_terminated(&content)?;

    Ok(elems.into_iter().collect())
}
