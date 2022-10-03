use super::{write::write, OOF_METHODS};
use proc_macro2::Span;
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    token::{Await, Brace, Dot, Eq, Let, Semi},
    Expr, ExprAwait, ExprCall, ExprMethodCall, Ident, Path, PathArguments, ReturnType,
};

pub fn context<'a>(tokens: &'a mut proc_macro2::TokenStream) -> Context<'a> {
    Context::new(tokens)
}

pub struct Context<'a> {
    tokens: &'a mut proc_macro2::TokenStream,
}

impl<'a> Context<'a> {
    fn new(tokens: &'a mut proc_macro2::TokenStream) -> Self {
        Self { tokens }
    }

    pub fn expr(self, expr: &Expr) {
        ContextInner::expr(expr).to_tokens(self.tokens);
    }
}

struct ContextInner<'a> {
    agg_index: usize,
    receiver: Receiver<'a>,
    chain: Vec<Method<'a>>,
    oof_methods: Vec<&'a ExprMethodCall>,
}

impl<'a> ToTokens for ContextInner<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            receiver,
            chain,
            oof_methods,
            agg_index: _,
        } = self;

        Brace(Span::call_site()).surround(tokens, |braced| {
            braced.extend(quote! {
                use ::oofs::used_by_attribute::*;
                let __display_owned = true && DISPLAY_OWNED;
            });

            receiver.write_prep(braced);

            for method in chain {
                method.write_prep(braced);
            }

            receiver.write_call(braced);

            for method in chain {
                method.write_call(braced);
            }

            let span = if let Some(last) = chain.last() {
                last.expr.span()
            } else {
                receiver.get_span()
            };

            braced.extend(quote_spanned! {span=>
                .with_oof_builder(|| {
                    let context = Context::new(#receiver .into())
                    #(
                        .with_method(#chain)
                    )* ;

                    OofBuilder::new(context.into())
                })
            });
        });

        for oof_method in oof_methods {
            let ExprMethodCall {
                dot_token,
                method,
                turbofish,
                paren_token,
                args,
                ..
            } = oof_method;
            dot_token.to_tokens(tokens);
            method.to_tokens(tokens);
            turbofish.to_tokens(tokens);
            paren_token.surround(tokens, |parens| {
                for pair in args.pairs() {
                    pair.value().to_tokens(parens);
                    pair.punct().to_tokens(parens);
                }
            });
        }

        tokens.extend(quote!(.map_err(|b| b.build())));
    }
}

impl<'a> ContextInner<'a> {
    fn expr(expr: &'a Expr) -> Self {
        let depth = 0;

        Self::_expr(expr, depth)
    }

    fn _expr(expr: &'a Expr, depth: usize) -> Self {
        match expr {
            Expr::MethodCall(call) => Self::_method_call(call, depth),
            Expr::Call(call) => Self::_call(call, depth),
            Expr::Await(expr_await) => Self::_await(expr_await, depth),
            Expr::Path(_) => Self::_path(expr, depth),
            expr => Self::_other(expr, depth),
        }
    }

    fn _method_call(_method_call: &'a ExprMethodCall, depth: usize) -> Self {
        // if the given method call is any of oof_methods like .tag(), .add_context(), etc.
        // then oof the receiver expr.
        if OOF_METHODS.iter().any(|m| _method_call.method == m) {
            let mut this = Self::_expr(&_method_call.receiver, depth);

            this.oof_methods.push(_method_call);

            return this;
        }

        let mut this = Self::_expr(&_method_call.receiver, depth + 1);

        let index = this.chain.len();

        let method = Method::new(index, &mut this.agg_index, _method_call);

        this.chain.push(method);

        this
    }

    fn _call(_call: &'a ExprCall, depth: usize) -> Self {
        let mut agg_index = 0;

        Self {
            receiver: Receiver::call(&mut agg_index, _call),
            agg_index,
            chain: Vec::with_capacity(depth),
            oof_methods: Vec::new(),
        }
    }

    fn _await(_await: &'a ExprAwait, depth: usize) -> Self {
        let ExprAwait {
            base,
            dot_token,
            await_token,
            ..
        } = _await;

        let mut this = Self::_expr(base, depth);

        if let Some(method) = this.chain.last_mut() {
            method.dot_await(dot_token, await_token);
        } else {
            this.receiver.dot_await(dot_token, await_token);
        }

        this
    }

    // Check if this is a variable ident.
    // If it is a variable, then it should not be evaluated.
    fn _path(expr: &'a Expr, depth: usize) -> Self {
        if let Expr::Path(path) = expr {
            if path.qself.is_none()
                && path.path.leading_colon.is_none()
                && path.path.segments.len() == 1
            {
                let first = path.path.segments.first().unwrap();
                if matches!(first.arguments, PathArguments::None) {
                    return Self {
                        receiver: Receiver::ident(&first.ident),
                        agg_index: 0,
                        chain: Vec::with_capacity(depth),
                        oof_methods: Vec::new(),
                    };
                }
            }
        }

        Self::_other(expr, depth)
    }

    fn _other(_other: &'a Expr, depth: usize) -> Self {
        let mut agg_index = 0;

        Self {
            receiver: Receiver::arg(&mut agg_index, _other),
            agg_index,
            chain: Vec::with_capacity(depth),
            oof_methods: Vec::new(),
        }
    }
}

struct DotAwait<'a> {
    dot_token: &'a Dot,
    await_token: &'a Await,
}

impl<'a> ToTokens for DotAwait<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.dot_token.to_tokens(tokens);
        self.await_token.to_tokens(tokens);
    }
}

enum Receiver<'a> {
    Ident(IdentReceiver<'a>),
    Call(Call<'a>),
    Arg(Arg<'a>),
}

impl<'a> Receiver<'a> {
    fn get_span(&self) -> Span {
        match self {
            Self::Ident(i) => i.get_span(),
            Self::Call(c) => c.get_span(),
            Self::Arg(a) => a.get_span(),
        }
    }

    fn ident(ident: &'a Ident) -> Self {
        Self::Ident(IdentReceiver::new(ident))
    }

    fn call(agg_index: &mut usize, expr: &'a ExprCall) -> Self {
        Self::Call(Call::new("__recv", agg_index, expr))
    }

    fn arg(agg_index: &mut usize, expr: &'a Expr) -> Self {
        Self::Arg(Arg::new("__recv", 0, agg_index, expr))
    }

    fn dot_await(&mut self, dot_token: &'a Dot, await_token: &'a Await) {
        match self {
            Self::Ident(i) => i.dot_await(dot_token, await_token),
            Self::Arg(a) => a.dot_await(dot_token, await_token),
            Self::Call(c) => c.dot_await(dot_token, await_token),
        }
    }

    fn write_prep(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Ident(i) => i.write_prep(tokens),
            Self::Arg(a) => a.write_prep(tokens),
            Self::Call(c) => c.write_prep(tokens),
        }
    }

    fn write_call(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Ident(i) => i.write_call(tokens),
            Self::Arg(a) => a.write_call(tokens),
            Self::Call(c) => c.write_call(tokens),
        }
    }
}

impl<'a> ToTokens for Receiver<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Ident(i) => i.to_tokens(tokens),
            Self::Arg(a) => a.to_tokens(tokens),
            Self::Call(c) => c.to_tokens(tokens),
        }
    }
}

struct IdentReceiver<'a> {
    ident: &'a Ident,
    dot_await: Option<DotAwait<'a>>,
}

impl<'a> IdentReceiver<'a> {
    fn new(ident: &'a Ident) -> Self {
        Self {
            ident,
            dot_await: None,
        }
    }

    fn dot_await(&mut self, dot_token: &'a Dot, await_token: &'a Await) {
        self.dot_await.replace(DotAwait {
            dot_token,
            await_token,
        });
    }

    fn write_prep(&self, _tokens: &mut proc_macro2::TokenStream) {}

    fn write_call(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { ident, dot_await } = self;

        ident.to_tokens(tokens);
        dot_await.to_tokens(tokens);
    }

    fn get_span(&self) -> Span {
        self.ident.span()
    }
}

impl<'a> ToTokens for IdentReceiver<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { ident, dot_await } = self;

        let is_async = dot_await.is_some();

        tokens.extend(quote! {
            Ident::new(#is_async, stringify!(#ident))
        });
    }
}

struct Call<'a> {
    name: String,
    args: Vec<(Arg<'a>, Option<&'a Comma>)>,
    dot_await: Option<DotAwait<'a>>,
    expr: &'a ExprCall,
}

impl<'a> Call<'a> {
    fn new(prefix: &str, agg_index: &mut usize, expr: &'a ExprCall) -> Self {
        let mut name = String::new();
        fmt_func(&mut name, &expr.func);

        let this = Self {
            name,
            dot_await: None,
            args: Arg::from_punctuated(prefix, agg_index, &expr.args),
            expr,
        };

        this
    }

    fn dot_await(&mut self, dot_token: &'a Dot, await_token: &'a Await) {
        self.dot_await.replace(DotAwait {
            dot_token,
            await_token,
        });
    }

    fn write_prep(&self, tokens: &mut proc_macro2::TokenStream) {
        for (arg, _) in &self.args {
            arg.write_prep(tokens);
        }
    }

    fn write_call(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            args,
            dot_await,
            expr,
            ..
        } = self;
        let ExprCall {
            func, paren_token, ..
        } = expr;

        write(tokens).expr(func);
        paren_token.surround(tokens, |parens| {
            for (arg, punct) in args {
                arg.write_call(parens);
                punct.to_tokens(parens);
            }
        });
        dot_await.to_tokens(tokens);
    }

    fn get_span(&self) -> Span {
        self.expr.span()
    }
}

impl<'a> ToTokens for Call<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            name,
            args,
            dot_await,
            ..
        } = self;

        let is_async = dot_await.is_some();

        let args = args.iter().map(|(a, _)| a);

        tokens.extend(quote! {
            Method::new(#is_async, #name, vec![#(#args),*])
        });
    }
}

struct Method<'a> {
    args: Vec<(Arg<'a>, Option<&'a Comma>)>,
    dot_await: Option<DotAwait<'a>>,
    expr: &'a ExprMethodCall,
}

impl<'a> Method<'a> {
    fn new(index: usize, agg_index: &mut usize, expr: &'a ExprMethodCall) -> Self {
        let prefix = format!("__{}", index);

        let this = Self {
            args: Arg::from_punctuated(&prefix, agg_index, &expr.args),
            dot_await: None,
            expr,
        };

        this
    }

    fn dot_await(&mut self, dot_token: &'a Dot, await_token: &'a Await) {
        self.dot_await.replace(DotAwait {
            dot_token,
            await_token,
        });
    }

    fn write_prep(&self, tokens: &mut proc_macro2::TokenStream) {
        for (arg, _) in &self.args {
            arg.write_prep(tokens);
        }
    }

    fn write_call(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            args,
            dot_await,
            expr,
        } = self;
        let ExprMethodCall {
            dot_token,
            method,
            turbofish,
            paren_token,
            ..
        } = expr;

        dot_token.to_tokens(tokens);
        method.to_tokens(tokens);
        turbofish.to_tokens(tokens);
        paren_token.surround(tokens, |parens| {
            for (arg, punct) in args {
                arg.write_call(parens);
                punct.to_tokens(parens);
            }
        });
        dot_await.to_tokens(tokens);
    }
}

impl<'a> ToTokens for Method<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            args,
            dot_await,
            expr,
            ..
        } = self;

        let is_async = dot_await.is_some();
        let name = &expr.method;
        let args = args.iter().map(|(a, _)| a);

        tokens.extend(quote! {
            Method::new(#is_async, stringify!(#name), vec![#(#args),*])
        });
    }
}

struct Arg<'a> {
    name: String,
    arg: Ident,
    arg_type: Ident,
    arg_bin: Ident,
    arg_ref_type: Ident,
    arg_display_fn: Ident,
    dot_await: Option<DotAwait<'a>>,
    expr: &'a Expr,
}

impl<'a> Arg<'a> {
    fn new(prefix: &str, index: usize, agg_index: &mut usize, expr: &'a Expr) -> Arg<'a> {
        let arg_str = format!("{}_{}", prefix, index);

        let name = agg_index.to_string();

        *agg_index += 1;

        Arg {
            name,
            arg: Ident::new(&arg_str, expr.span()),
            arg_type: Ident::new(&format!("{arg_str}_type"), expr.span()),
            arg_bin: Ident::new(&format!("{arg_str}_bin"), expr.span()),
            arg_ref_type: Ident::new(&format!("{arg_str}_ref_type"), expr.span()),
            arg_display_fn: Ident::new(&format!("{arg_str}_display_fn"), expr.span()),
            dot_await: None,
            expr,
        }
    }

    fn from_punctuated(
        prefix: &str,
        agg_index: &mut usize,
        puntuated: &'a Punctuated<Expr, Comma>,
    ) -> Vec<(Arg<'a>, Option<&'a Comma>)> {
        puntuated
            .pairs()
            .enumerate()
            .map(|(i, a)| {
                (
                    Arg::new(prefix, i, agg_index, a.value()),
                    a.punct().map(|p| *p),
                )
            })
            .collect()
    }

    fn dot_await(&mut self, dot_token: &'a Dot, await_token: &'a Await) {
        self.dot_await.replace(DotAwait {
            dot_token,
            await_token,
        });
    }

    fn write_prep(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            arg,
            arg_type,
            arg_bin,
            arg_ref_type,
            arg_display_fn,
            expr,
            ..
        } = self;

        Let(Span::call_site()).to_tokens(tokens);
        arg.to_tokens(tokens);
        Eq(Span::call_site()).to_tokens(tokens);
        write(tokens).expr(expr);
        Semi(Span::call_site()).to_tokens(tokens);

        tokens.extend(quote! {
            let #arg_type = #arg.__type_name();
            let #arg_bin = __TsaBin(#arg);
            let #arg_ref_type = #arg_bin.__ref_type();
            let #arg_display_fn = #arg_bin.__try_lazy_fn(__display_owned, |v| v.__try_debug());
            let #arg = #arg_bin.__tsa_unload();
        });
    }

    fn write_call(&self, tokens: &mut proc_macro2::TokenStream) {
        self.arg.to_tokens(tokens);
        self.dot_await.to_tokens(tokens);
    }

    fn get_span(&self) -> Span {
        self.expr.span()
    }
}

impl<'a> ToTokens for Arg<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            name,
            arg_type,
            arg_ref_type,
            arg_display_fn,
            ..
        } = self;

        tokens.extend(quote! {
            Arg::new(
                #name,
                #arg_ref_type,
                #arg_type,
                #arg_display_fn.call(),
            )
        });
    }
}

fn fmt_func(f: &mut String, func: &Expr) {
    use Expr::*;
    match func {
        Path(path) => {
            if path.qself.is_some() {
                *f += "<_>::";
            }
            fmt_path(f, &path.path);
        }
        Async(e) => {
            *f += &format!("async {}{{ ... }}", e.capture.map(|_| "move").unwrap_or(""));
        }
        Await(e) => {
            fmt_func(f, e.base.as_ref());
            *f += ".await";
        }
        Binary(e) => {
            fmt_func(f, &e.left);
            *f += &format!(" {} ", e.op.to_token_stream().to_string());
            fmt_func(f, &e.right);
        }
        Box(e) => {
            *f += "box ";
            fmt_func(f, &e.expr);
        }
        Break(e) => {
            *f += "break ";
            if let Some(expr) = &e.expr {
                fmt_func(f, &expr);
            }
        }
        Call(e) => {
            fmt_func(f, &e.func);
            f.push('(');
            for a in e.args.pairs() {
                *f += "_";
                if a.punct().is_some() {
                    *f += ", ";
                }
            }
            f.push(')');
        }
        Block(_) => *f += "{ ... }",
        Cast(e) => {
            fmt_func(f, &e.expr);
            *f += " as _";
        }
        Closure(e) => {
            if e.movability.is_some() {
                *f += "static ";
            }
            if e.asyncness.is_some() {
                *f += "async ";
            }
            if e.capture.is_some() {
                *f += "move ";
            }
            *f += "|";
            for a in e.inputs.pairs() {
                *f += "_";
                if a.punct().is_some() {
                    *f += ", ";
                }
            }
            *f += "| { ... }";
        }
        Field(e) => {
            fmt_func(f, &e.base);
            *f += &format!(".{}", e.member.to_token_stream());
        }
        ForLoop(e) => {
            *f += "for _ in ";
            fmt_func(f, &e.expr);
            *f += " { ... }";
        }
        Group(e) => fmt_func(f, &e.expr),
        If(e) => {
            *f += "if ";
            fmt_func(f, &e.cond);
            *f += " { ... }";
            if e.else_branch.is_some() {
                *f += " else { ... }";
            }
        }
        Index(e) => {
            fmt_func(f, &e.expr);
            *f += "[";
            fmt_func(f, &e.index);
            *f += "]";
        }
        Let(e) => {
            *f += "let _ = ";
            fmt_func(f, &e.expr);
        }
        Lit(e) => *f += &e.to_token_stream().to_string(),
        Loop(_) => *f += "loop { ... }",
        Macro(e) => {
            use syn::MacroDelimiter::*;
            fmt_path(f, &e.mac.path);
            *f += "!";
            match &e.mac.delimiter {
                Paren(_) => *f += "(...)",
                Brace(_) => *f += "{...}",
                Bracket(_) => *f += "[...]",
            }
        }
        Match(_) => *f += "match { ... }",
        MethodCall(e) => {
            fmt_func(f, &e.receiver);
            *f += ".";
            *f += &e.method.to_string();
            if let Some(t) = &e.turbofish {
                *f += "::<";
                for a in t.args.pairs() {
                    *f += "_";
                    if a.punct().is_some() {
                        *f += ", ";
                    }
                }
                *f += ">";
            }
            *f += "(";
            for a in e.args.pairs() {
                *f += "_";
                if a.punct().is_some() {
                    *f += ", ";
                }
            }
            *f += ")";
        }
        Paren(e) => {
            *f += "(";
            fmt_func(f, &e.expr);
            *f += ")";
        }
        Reference(e) => {
            *f += "&";
            if e.mutability.is_some() {
                *f += "mut ";
            }
            fmt_func(f, &e.expr);
        }
        Array(e) => {
            *f += "[";
            for pair in e.elems.pairs() {
                fmt_func(f, &pair.value());
                if pair.punct().is_some() {
                    *f += ", ";
                }
            }
            *f += "]";
        }
        Repeat(e) => {
            *f += "[";
            fmt_func(f, &e.expr);
            *f += "; ";
            fmt_func(f, &e.len);
            *f += "]";
        }
        Struct(e) => {
            fmt_path(f, &e.path);
            *f += "{ ... }";
        }
        Try(e) => {
            fmt_func(f, &e.expr);
            *f += "?";
        }
        TryBlock(_) => *f += "try { ... }",
        Tuple(e) => {
            *f += "(";
            for pair in e.elems.pairs() {
                fmt_func(f, &pair.value());
                if pair.punct().is_some() {
                    *f += ", ";
                }
            }
            *f += ")";
        }
        Type(e) => {
            fmt_func(f, &e.expr);
            *f += ": _";
        }
        Unary(e) => {
            *f += &e.op.to_token_stream().to_string();
            fmt_func(f, &e.expr);
        }
        Unsafe(_) => *f += "unsafe { ... }",
        While(e) => {
            *f += "while ";
            fmt_func(f, &e.cond);
            *f += " { ... }";
        }
        Yield(e) => {
            *f += "yield ";
            if let Some(expr) = &e.expr {
                fmt_func(f, &expr);
            }
        }
        _ => *f += "_",
    }
}

fn fmt_path(f: &mut String, path: &Path) {
    if path.leading_colon.is_some() {
        *f += "::";
    }
    for pair in path.segments.pairs() {
        let path = pair.value();
        *f += &path.ident.to_string();
        match &path.arguments {
            PathArguments::Parenthesized(a) => {
                *f += "(";
                for pair in a.inputs.pairs() {
                    *f += "_";
                    if pair.punct().is_some() {
                        *f += ", ";
                    }
                }
                *f += ")";
                if matches!(a.output, ReturnType::Type(_, _)) {
                    *f += " -> _";
                }
            }
            PathArguments::AngleBracketed(a) => {
                if a.colon2_token.is_some() {
                    *f += "::";
                }
                *f += "<";
                for pair in a.args.pairs() {
                    *f += "_";
                    if pair.punct().is_some() {
                        *f += ", ";
                    }
                }
                *f += ">";
            }
            PathArguments::None => {}
        }
        if pair.punct().is_some() {
            *f += "::";
        }
    }
}
