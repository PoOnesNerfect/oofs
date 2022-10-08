use super::props::Properties;
use proc_macro2::Span;
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    token::{Await, Brace, Dot, Eq, Let, Paren, Semi},
    Expr, ExprAwait, ExprCall, ExprField, ExprMethodCall, Ident, Path, PathArguments, ReturnType,
};

pub struct Context<'a> {
    tokens: &'a mut proc_macro2::TokenStream,
    props: Properties,
    depth: usize,
}

impl<'a> Context<'a> {
    pub fn new(tokens: &'a mut proc_macro2::TokenStream, props: Properties) -> Self {
        Self {
            tokens,
            props,
            depth: 0,
        }
    }

    pub fn expr(mut self, expr: &'a Expr) {
        let inner = self._expr(expr);
        inner.to_tokens(self.tokens, self.props);
    }

    fn _expr(&mut self, expr: &'a Expr) -> ContextInner<'a> {
        match expr {
            Expr::MethodCall(call) => self._method_call(call),
            Expr::Call(call) => self._call(call),
            Expr::Await(expr_await) => self._await(expr_await),
            Expr::Path(_) => self._path(expr),
            Expr::Field(_) => self._field(expr),
            expr => self._other(expr),
        }
    }

    fn _method_call(&mut self, _method_call: &'a ExprMethodCall) -> ContextInner<'a> {
        self.depth += 1;

        let mut this = self._expr(&_method_call.receiver);

        let index = this.chain.len();

        let method = Method::new(index, &mut this.agg_index, _method_call, self.props.clone());

        this.chain.push(method);

        this
    }

    fn _call(&mut self, _call: &'a ExprCall) -> ContextInner<'a> {
        ContextInner::call(_call, self.depth, self.props.clone())
    }

    fn _await(&mut self, _await: &'a ExprAwait) -> ContextInner<'a> {
        let ExprAwait {
            base,
            dot_token,
            await_token,
            ..
        } = _await;

        let mut this = self._expr(base);

        if let Some(method) = this.chain.last_mut() {
            method.dot_await(dot_token, await_token);
        } else {
            this.receiver.dot_await(dot_token, await_token);
        }

        this
    }

    // Check if this is a variable ident.
    // If it is a variable, then it should not be evaluated.
    fn _path(&mut self, expr: &'a Expr) -> ContextInner<'a> {
        if let Expr::Path(path) = expr {
            if path.qself.is_none() {
                if let Some(ident) = path.path.get_ident() {
                    return ContextInner::ident(ident, self.depth);
                }
            }
        }

        self._other(expr)
    }

    fn _field(&mut self, expr: &'a Expr) -> ContextInner<'a> {
        if let Expr::Field(field) = expr {
            // if field is of a variable, we don't want to consume it.
            if matches!(field.base.as_ref(), Expr::Path(_)) {
                return ContextInner::field(field, self.depth);
            }
        }

        self._other(expr)
    }

    fn _other(&mut self, _other: &'a Expr) -> ContextInner<'a> {
        ContextInner::arg(_other, self.depth, self.props.clone())
    }
}

struct ContextInner<'a> {
    agg_index: usize,
    receiver: Receiver<'a>,
    chain: Vec<Method<'a>>,
}

impl<'a> ContextInner<'a> {
    fn field(field: &'a ExprField, depth: usize) -> Self {
        Self {
            receiver: Receiver::field(field),
            chain: Vec::with_capacity(depth),
            agg_index: 0,
        }
    }

    fn ident(ident: &'a Ident, depth: usize) -> Self {
        Self {
            receiver: Receiver::ident(ident),
            chain: Vec::with_capacity(depth),
            agg_index: 0,
        }
    }

    fn call(expr: &'a ExprCall, depth: usize, props: Properties) -> Self {
        let mut agg_index = 0;

        ContextInner {
            receiver: Receiver::call(&mut agg_index, expr, props),
            agg_index,
            chain: Vec::with_capacity(depth),
        }
    }

    fn arg(expr: &'a Expr, depth: usize, props: Properties) -> Self {
        let mut agg_index = 0;

        ContextInner {
            receiver: Receiver::arg(&mut agg_index, expr, props),
            agg_index,
            chain: Vec::with_capacity(depth),
        }
    }

    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream, props: Properties) {
        let Self {
            receiver,
            chain,
            agg_index: _,
        } = self;

        Brace(Span::call_site()).surround(tokens, |braced| {
            braced.extend(quote! {
                use ::oofs::__used_by_attribute::*;
                let __display_owned = DEBUG_OWNED;

                fn type_name_of_val<T>(_t: &T) -> &'static str {
                    core::any::type_name::<T>()
                }
            });

            receiver.write_prep(braced);

            for method in chain.iter().filter(|m| !m.is_meta) {
                method.write_prep(braced);
            }

            let span = if let Some(last) = chain.last() {
                last.expr.span()
            } else {
                receiver.get_span()
            };

            braced.extend(quote_spanned!(span=> OofGenerator::build_oof));
            Paren(span).surround(braced, |parens| {
                receiver.write_call(parens);

                for method in chain {
                    method.write_call(parens);
                }

                for tag in &props.tags {
                    parens.extend(quote!(._tag::<#tag>()));
                }

                for attach in &props.attach {
                    parens.extend(quote!(._attach(#attach)));
                }

                for lazy in &props.attach_lazy {
                    parens.extend(quote!(._attach_lazy(#lazy)));
                }

                parens
                    .extend(quote_spanned!(span=>, || OofGeneratedContext::new(#receiver.into())));

                // if the given method call is a meta method, then skip creating a context.
                for method in chain.iter().filter(|m| !m.is_meta) {
                    parens.extend(quote_spanned!(span=>.with_method(#method)));
                }
            });
        });
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
    Field(Field<'a>),
    Call(Call<'a>),
    Arg(Arg<'a>),
}

impl<'a> Receiver<'a> {
    fn get_span(&self) -> Span {
        match self {
            Self::Ident(i) => i.get_span(),
            Self::Field(f) => f.get_span(),
            Self::Call(c) => c.get_span(),
            Self::Arg(a) => a.get_span(),
        }
    }

    fn field(field: &'a ExprField) -> Self {
        Self::Field(Field::new(field))
    }

    fn ident(ident: &'a Ident) -> Self {
        Self::Ident(IdentReceiver::new(ident))
    }

    fn call(agg_index: &mut usize, expr: &'a ExprCall, props: Properties) -> Self {
        Self::Call(Call::new("__recv", agg_index, expr, props))
    }

    fn arg(agg_index: &mut usize, expr: &'a Expr, props: Properties) -> Self {
        Self::Arg(Arg::new("__recv", 0, agg_index, expr, props))
    }

    fn dot_await(&mut self, dot_token: &'a Dot, await_token: &'a Await) {
        match self {
            Self::Ident(i) => i.dot_await(dot_token, await_token),
            Self::Field(f) => f.dot_await(dot_token, await_token),
            Self::Arg(a) => a.dot_await(dot_token, await_token),
            Self::Call(c) => c.dot_await(dot_token, await_token),
        }
    }

    fn write_prep(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Ident(i) => i.write_prep(tokens),
            Self::Field(f) => f.write_prep(tokens),
            Self::Arg(a) => a.write_prep(tokens),
            Self::Call(c) => c.write_prep(tokens),
        }
    }

    fn write_call(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Ident(i) => i.write_call(tokens),
            Self::Field(f) => f.write_call(tokens),
            Self::Arg(a) => a.write_call(tokens),
            Self::Call(c) => c.write_call(tokens),
        }
    }
}

impl<'a> ToTokens for Receiver<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Ident(i) => i.to_tokens(tokens),
            Self::Field(f) => f.to_tokens(tokens),
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
            OofIdent::new(#is_async, stringify!(#ident))
        });
    }
}
struct Field<'a> {
    field: &'a ExprField,
    dot_await: Option<DotAwait<'a>>,
}

impl<'a> Field<'a> {
    fn new(field: &'a ExprField) -> Self {
        Self {
            field,
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
        let Self { field, dot_await } = self;

        field.to_tokens(tokens);
        dot_await.to_tokens(tokens);
    }

    fn get_span(&self) -> Span {
        self.field.span()
    }
}

impl<'a> ToTokens for Field<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { field, dot_await } = self;

        let is_async = dot_await.is_some();

        tokens.extend(quote! {
            OofIdent::new(#is_async, stringify!(#field))
        });
    }
}

struct Call<'a> {
    name: String,
    args: Vec<(Arg<'a>, Option<&'a Comma>)>,
    dot_await: Option<DotAwait<'a>>,
    expr: &'a ExprCall,
    props: Properties,
}

impl<'a> Call<'a> {
    fn new(prefix: &str, agg_index: &mut usize, expr: &'a ExprCall, props: Properties) -> Self {
        let mut name = String::new();
        fmt_expr(&mut name, &expr.func);

        let this = Self {
            name,
            dot_await: None,
            args: Arg::from_punctuated(prefix, agg_index, &expr.args, props.clone()),
            expr,
            props,
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
            props,
            ..
        } = self;
        let ExprCall {
            func, paren_token, ..
        } = expr;

        props.write(tokens).expr(func);
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
            OofMethod::new(#is_async, #name, vec![#(#args),*])
        });
    }
}

struct Method<'a> {
    is_meta: bool,
    args: Vec<(Arg<'a>, Option<&'a Comma>)>,
    dot_await: Option<DotAwait<'a>>,
    expr: &'a ExprMethodCall,
    props: Properties,
}

impl<'a> Method<'a> {
    fn new(
        index: usize,
        agg_index: &mut usize,
        expr: &'a ExprMethodCall,
        props: Properties,
    ) -> Self {
        let prefix = format!("__{}", index);

        let is_meta = expr.method.to_string().starts_with('_');

        let args = (!is_meta)
            .then(|| Arg::from_punctuated(&prefix, agg_index, &expr.args, props.clone()))
            .unwrap_or_default();

        let this = Self {
            is_meta,
            args,
            dot_await: None,
            expr,
            props,
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
            is_meta,
            args,
            dot_await,
            expr,
            props,
        } = self;
        let ExprMethodCall {
            dot_token,
            method,
            turbofish,
            paren_token,
            args: meta_args,
            ..
        } = expr;

        dot_token.to_tokens(tokens);
        method.to_tokens(tokens);
        turbofish.to_tokens(tokens);
        paren_token.surround(tokens, |parens| {
            if *is_meta {
                for pair in meta_args.pairs() {
                    props.write(parens).expr(pair.value());
                    pair.punct().to_tokens(parens);
                }
            } else {
                for (arg, punct) in args {
                    arg.write_call(parens);
                    punct.to_tokens(parens);
                }
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
            OofMethod::new(#is_async, stringify!(#name), vec![#(#args),*])
        });
    }
}

struct Arg<'a> {
    index: usize,
    arg: Ident,
    arg_type: Ident,
    arg_bin: Ident,
    arg_display_fn: Ident,
    dot_await: Option<DotAwait<'a>>,
    expr: &'a Expr,
    props: Properties,
}

impl<'a> Arg<'a> {
    fn new(
        prefix: &str,
        index: usize,
        agg_index: &mut usize,
        expr: &'a Expr,
        props: Properties,
    ) -> Arg<'a> {
        let arg_str = format!("{}_{}", prefix, index);

        let index = *agg_index;

        *agg_index += 1;

        Arg {
            index,
            arg: Ident::new(&arg_str, expr.span()),
            arg_type: Ident::new(&format!("{arg_str}_type"), expr.span()),
            arg_bin: Ident::new(&format!("{arg_str}_bin"), expr.span()),
            arg_display_fn: Ident::new(&format!("{arg_str}_display_fn"), expr.span()),
            dot_await: None,
            expr,
            props,
        }
    }

    fn from_punctuated(
        prefix: &str,
        agg_index: &mut usize,
        puntuated: &'a Punctuated<Expr, Comma>,
        props: Properties,
    ) -> Vec<(Arg<'a>, Option<&'a Comma>)> {
        puntuated
            .pairs()
            .enumerate()
            .map(|(i, a)| {
                (
                    Arg::new(prefix, i, agg_index, a.value(), props.clone()),
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
            arg_display_fn,
            expr,
            props,
            ..
        } = self;

        Let(Span::call_site()).to_tokens(tokens);
        arg.to_tokens(tokens);
        Eq(Span::call_site()).to_tokens(tokens);
        props.write(tokens).expr(expr);
        Semi(Span::call_site()).to_tokens(tokens);

        tokens.extend(quote! {
            let #arg_type = type_name_of_val(&#arg);
            let #arg_bin = __TsaBin(#arg);
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
            index,
            arg_type,
            arg_display_fn,
            ..
        } = self;

        tokens.extend(quote! {
            OofArg::new(
                #index,
                #arg_type,
                #arg_display_fn.call(),
            )
        });
    }
}

fn fmt_expr(f: &mut String, func: &Expr) {
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
            fmt_expr(f, e.base.as_ref());
            *f += ".await";
        }
        Binary(e) => {
            fmt_expr(f, &e.left);
            *f += &format!(" {} ", e.op.to_token_stream().to_string());
            fmt_expr(f, &e.right);
        }
        Box(e) => {
            *f += "box ";
            fmt_expr(f, &e.expr);
        }
        Break(e) => {
            *f += "break ";
            if let Some(expr) = &e.expr {
                fmt_expr(f, &expr);
            }
        }
        Call(e) => {
            fmt_expr(f, &e.func);
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
            fmt_expr(f, &e.expr);
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
            fmt_expr(f, &e.base);
            *f += &format!(".{}", e.member.to_token_stream());
        }
        ForLoop(e) => {
            *f += "for _ in ";
            fmt_expr(f, &e.expr);
            *f += " { ... }";
        }
        Group(e) => fmt_expr(f, &e.expr),
        If(e) => {
            *f += "if ";
            fmt_expr(f, &e.cond);
            *f += " { ... }";
            if e.else_branch.is_some() {
                *f += " else { ... }";
            }
        }
        Index(e) => {
            fmt_expr(f, &e.expr);
            *f += "[";
            fmt_expr(f, &e.index);
            *f += "]";
        }
        Let(e) => {
            *f += "let _ = ";
            fmt_expr(f, &e.expr);
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
            fmt_expr(f, &e.receiver);
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
            fmt_expr(f, &e.expr);
            *f += ")";
        }
        Reference(e) => {
            *f += "&";
            if e.mutability.is_some() {
                *f += "mut ";
            }
            fmt_expr(f, &e.expr);
        }
        Array(e) => {
            *f += "[";
            for pair in e.elems.pairs() {
                fmt_expr(f, &pair.value());
                if pair.punct().is_some() {
                    *f += ", ";
                }
            }
            *f += "]";
        }
        Repeat(e) => {
            *f += "[";
            fmt_expr(f, &e.expr);
            *f += "; ";
            fmt_expr(f, &e.len);
            *f += "]";
        }
        Struct(e) => {
            fmt_path(f, &e.path);
            *f += "{ ... }";
        }
        Try(e) => {
            fmt_expr(f, &e.expr);
            *f += "?";
        }
        TryBlock(_) => *f += "try { ... }",
        Tuple(e) => {
            *f += "(";
            for pair in e.elems.pairs() {
                fmt_expr(f, &pair.value());
                if pair.punct().is_some() {
                    *f += ", ";
                }
            }
            *f += ")";
        }
        Type(e) => {
            fmt_expr(f, &e.expr);
            *f += ": _";
        }
        Unary(e) => {
            *f += &e.op.to_token_stream().to_string();
            fmt_expr(f, &e.expr);
        }
        Unsafe(_) => *f += "unsafe { ... }",
        While(e) => {
            *f += "while ";
            fmt_expr(f, &e.cond);
            *f += " { ... }";
        }
        Yield(e) => {
            *f += "yield ";
            if let Some(expr) = &e.expr {
                fmt_expr(f, &expr);
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
