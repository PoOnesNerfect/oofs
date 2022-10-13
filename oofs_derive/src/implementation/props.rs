use super::{context::Context, write::Writer};
use proc_macro2::{Group, Spacing, TokenStream, TokenTree};
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use std::{
    iter::once,
    ops::{Deref, DerefMut},
};
use syn::{
    buffer::Cursor, parenthesized, parse::Parse, punctuated::Punctuated, token::Paren, Attribute,
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
        use syn::spanned::Spanned;

        let Attribute { path, tokens, .. } = attr;
        if !path.is_ident("oofs") {
            return false;
        }

        if !tokens.is_empty() {
            match syn::parse2(tokens.clone()) {
                Ok(parsed) => self.merge(parsed),
                Err(e) => {
                    abort!(tokens.span(), "{}", e.to_string());
                }
            }
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

macro_rules! impl_prop_args {
    ($($f:ident : $t:ty $(as $wrap:ident)?),* $(,)?) => {
        #[derive(Clone)]
        pub struct PropArgs {
            $(
                pub $f: impl_prop_args!(@ty $t $(as $wrap)?),
            )*
        }

        paste::paste!{
            impl Default for PropArgs {
                fn default() -> Self {
                    Self {
                        $(
                            $f: impl_prop_args!(@default [<$t>] $(as $wrap)?),
                        )*
                    }
                }
            }

            impl PropArgs {
                pub fn merge(&mut self, other: PropArgs) {
                    $(
                        impl_prop_args!(@merge self.$f => other.$f => [<$t>] $(as $wrap)?);
                    )*
                }

                $(
                    impl_prop_args!(@helper $f => [<$t>] $(as $wrap)?);
                )*
            }

            impl Parse for PropArgs {
                fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                    let parsed: Punctuated<Arg, Token!(,)> = Punctuated::parse_terminated(input)?;

                    let mut args = PropArgs::default();

                    for arg in parsed {
                        use Arg::*;
                        match arg {
                            $(
                                [<$f:camel>](t) => args.$f = t,
                            )*
                        }
                    }

                    Ok(args)
                }
            }

            #[derive(Clone)]
            enum Arg {
                $(
                    [<$f:camel>](impl_prop_args!(@ty $t $(as $wrap)?)),
                )*
            }

            impl Parse for Arg {
                fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                    let ident: Ident = input.parse()?;

                    let arg = match ident {
                        $(
                            ident if ident == stringify!($f) => {
                                Self:: [<$f:camel>] (impl_prop_args!(@extract input => [<$t>] $(as $wrap)?)?)
                            },
                        )*
                        _ => {
                            abort!(ident, "Expected one of `{}`", vec![$(stringify!($f)),*].join("`, `"));
                        }
                    };

                    Ok(arg)
                }
            }
        }
    };
    (@ty $t:ty) => ($t);
    (@ty $t:ty as option) => (Option<$t>);
    (@ty $t:ty as vec) => (Vec<$t>);
    (@default $t:ident) => (Default::default());
    (@default $t:ident as option) => (None);
    (@default $t:ident as vec) => (Vec::new());
    (@merge $this:expr => $other:expr => bool) => ($this |= $other);
    (@merge $this:expr => $other:expr => $t:ident) => ($this = $other);
    (@merge $this:expr => $other:expr => $t:ident as option) => {
        if let Some(other) = $other {
            $this.replace(other);
        }
    };
    (@merge $this:expr => $other:expr => $t:ident as vec) => ($this.extend($other));
    (@helper $f:ident => bool as option) => {
        pub fn $f (&self) -> bool {
            self.$f.unwrap_or(false)
        }
    };
    (@helper $f:ident => $t:ident $(as $wrap:ident)?) => {
    };
    (@extract $input:expr => bool as option) => (extract_bool($input));
    (@extract $input:expr => $t:ident) => (extract_generic($input));
    (@extract $input:expr => $t:ident as option) => (extract_optional($input));
    (@extract $input:expr => $t:ident as vec) => (extract_vec($input));
}

impl_prop_args! {
    closures: bool as option,
    async_blocks: bool as option,
    skip: bool as option,
    tag: Type as vec,
    attach: Expr as vec,
    attach_lazy: Expr as vec,
    debug_skip: Expr as vec,
    debug_with: DebugWith as vec,
    debug_non_copyable: DebugNonCopyable,
}

#[derive(Clone, Copy)]
pub enum DebugNonCopyable {
    Full,
    Disabled,
    None,
    // CloneLazy,
}

impl Default for DebugNonCopyable {
    fn default() -> Self {
        DebugNonCopyable::None
    }
}

impl ToTokens for DebugNonCopyable {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use DebugNonCopyable::*;
        match self {
            Full => true.to_tokens(tokens),
            Disabled => false.to_tokens(tokens),
            None => tokens.extend(quote!(DEBUG_NON_COPYABLE)),
        }
    }
}

impl Parse for DebugNonCopyable {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;

        if ident == "disabled" {
            Ok(DebugNonCopyable::Disabled)
        } else if ident == "full" {
            Ok(DebugNonCopyable::Full)
        } else {
            abort!(ident, "Expected 'disabled' or 'full'");
        }
    }
}

#[derive(Clone)]
pub struct DebugWith {
    pub arg: Expr,
    pub debug_fn: Expr,
}

impl ToTokens for DebugWith {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.debug_fn.to_tokens(tokens);
    }
}

impl Parse for DebugWith {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut arg = TokenStream::new();
        let mut debug_fn = TokenStream::new();

        input.step(|cursor| {
            let mut cursor = *cursor;

            get_arg(&mut cursor, &mut arg);
            get_method(&mut cursor, &mut debug_fn);

            Ok(((), cursor))
        })?;

        Ok(Self {
            arg: syn::parse2(arg)?,
            debug_fn: syn::parse2(debug_fn)?,
        })
    }
}

fn get_arg(cursor: &mut Cursor, recv: &mut TokenStream) {
    while let Some((tt, next)) = cursor.token_tree() {
        if let TokenTree::Punct(p) = &tt {
            if p.as_char() == '-' && p.spacing() == Spacing::Joint {
                if let Some((tt2, next2)) = next.token_tree() {
                    if let TokenTree::Punct(p2) = &tt2 {
                        if p2.as_char() == '>' && p2.spacing() == Spacing::Alone {
                            *cursor = next2;
                            return;
                        }
                    }
                }
            }
        }

        recv.extend(once(tt));
        *cursor = next;
    }
}

fn get_method(cursor: &mut Cursor, recv: &mut TokenStream) {
    let mut found = false;
    let find = ('$', "a");
    let replace = quote!(v.target());

    while let Some((tt, next)) = cursor.token_tree() {
        if !found {
            match &tt {
                TokenTree::Group(g) => {
                    let (stream, found2) = find_and_replace(g.stream(), &find, &replace);

                    let g: TokenTree = Group::new(g.delimiter(), stream).into();
                    found = found2;

                    recv.extend(once(g));
                    *cursor = next;
                    continue;
                }
                TokenTree::Punct(p) => {
                    if p.as_char() == find.0 {
                        if let Some((tt2, next2)) = next.token_tree() {
                            if let TokenTree::Ident(i) = &tt2 {
                                if i == find.1 {
                                    recv.extend(once(replace.clone()));
                                    *cursor = next2;
                                    found = true;
                                    continue;
                                }
                            }

                            recv.extend([tt, tt2]);
                            continue;
                        }
                    }
                }
                _ => {}
            }
        }

        recv.extend(once(tt));
        *cursor = next;
    }
}

fn find_and_replace(
    tokens: TokenStream,
    find: &(char, &str),
    replace: &TokenStream,
) -> (TokenStream, bool) {
    let mut ret = TokenStream::new();
    let mut found = false;

    let mut tokens = tokens.into_iter();
    while let Some(tt) = tokens.next() {
        if found {
            ret.extend(tokens);
            break;
        }

        match &tt {
            TokenTree::Group(g) => {
                let (stream, found2) = find_and_replace(g.stream(), find, replace);
                found = found2;

                let g: TokenTree = Group::new(g.delimiter(), stream).into();
                ret.extend(once(g));
            }
            TokenTree::Punct(p) => {
                if p.as_char() == find.0 {
                    if let Some(tt2) = tokens.next() {
                        if let TokenTree::Ident(i) = &tt2 {
                            if i == find.1 {
                                ret.extend(once(replace.clone()));
                                ret.extend(tokens);
                                found = true;
                                break;
                            }
                        }

                        ret.extend([tt, tt2]);
                        continue;
                    }
                }
            }
            _ => {}
        }

        ret.extend(once(tt));
    }

    (ret, found)
}

fn extract_bool(input: syn::parse::ParseStream) -> syn::Result<Option<bool>> {
    // if argument is mentioned, like `#[oofs(closures)]`, it must enable this feature.
    let mut b = Some(true);

    if input.peek(Paren) {
        let content;
        parenthesized!(content in input);

        let token: LitBool = content.parse()?;
        b.replace(token.value());
    }

    Ok(b)
}

fn extract_generic<T: Default + Parse>(input: syn::parse::ParseStream) -> syn::Result<T> {
    let mut t = Default::default();

    if input.peek(Paren) {
        let content;
        parenthesized!(content in input);

        t = content.parse()?;
    }

    Ok(t)
}

fn extract_vec<T: Parse>(input: syn::parse::ParseStream) -> syn::Result<Vec<T>> {
    let content;
    parenthesized!(content in input);

    let elems: Punctuated<T, Token!(,)> = Punctuated::parse_terminated(&content)?;

    Ok(elems.into_iter().collect())
}
