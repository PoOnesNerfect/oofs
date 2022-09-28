use super::{write::write, OOF_METHODS};
use proc_macro2::{Ident, Span};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    token::{Await, Brace, Dot, Semi},
    Expr, ExprAwait, ExprCall, ExprMethodCall, ExprTry,
};

pub fn oof<'a>(tokens: &'a mut proc_macro2::TokenStream) -> Oof<'a> {
    Oof::new(tokens)
}

pub struct Oof<'a> {
    tokens: &'a mut proc_macro2::TokenStream,
    dot_await: Option<DotAwait<'a>>,
    should_build: Option<proc_macro2::TokenStream>,
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

impl<'a> Oof<'a> {
    fn new(tokens: &'a mut proc_macro2::TokenStream) -> Self {
        Oof {
            tokens,
            dot_await: None,
            should_build: None,
        }
    }

    pub fn dot_await(mut self, dot_token: &'a Dot, await_token: &'a Await) -> Self {
        self.dot_await.replace(DotAwait {
            dot_token,
            await_token,
        });
        self
    }

    pub fn should_build(mut self) -> Self {
        self.should_build.replace(quote!(.map_err(|b| b.build())));
        self
    }

    pub fn expr(self, expr: &Expr) {
        match expr {
            Expr::MethodCall(call) => self._method_call(call),
            Expr::Call(call) => self._call(call),
            Expr::Await(expr_await) => self._await(expr_await),
            Expr::Try(expr_try) => self._try(expr_try),
            expr => self._other(expr),
        }
    }

    fn _method_call(self, call: &ExprMethodCall) {
        let Oof {
            tokens,
            dot_await,
            should_build,
        } = self;

        let ExprMethodCall {
            attrs,
            receiver,
            method,
            turbofish,
            args,
            dot_token,
            paren_token,
        } = call;

        // if the given method call is any of oof_methods like .tag(), .add_context(), etc.
        // then oof the receiver expr.
        if OOF_METHODS.iter().any(|m| method == m) {
            for attr in attrs {
                attr.to_tokens(tokens);
            }

            oof(tokens).expr(receiver);

            dot_token.to_tokens(tokens);
            method.to_tokens(tokens);
            turbofish.to_tokens(tokens);
            paren_token.surround(tokens, |parens| {
                for pair in args.pairs() {
                    write(parens).expr(pair.value());
                    pair.punct().to_tokens(parens);
                }
            });
            dot_await.to_tokens(tokens);
            should_build.to_tokens(tokens);

            return;
        }

        let method_span = method.span();

        Brace(method_span).surround(tokens, |braced| {
            let is_async = dot_await.is_some();

            // FnModel =>
            braced.extend(quote_spanned! {method_span=>
                use oofs::used_by_attribute::*;

                let __fn_is_async = #is_async;
                let __fn_display_owned = false || DISPLAY_OWNED;
                let __fn_name = stringify!(#method);

                let __recv_var =
            });

            write(braced).expr(receiver);
            Semi(method_span).to_tokens(braced);

            braced.extend(quote_spanned! {method_span=>
                let __recv_type = __recv_var.__type_name();
                let __recv_bin = __TsaBin(__recv_var);
                let __recv_ref_type = __recv_bin.__ref_type();
                let __recv_fn = __recv_bin.__try_lazy_fn(__fn_display_owned, |v| v.__try_debug());
                let __recv_unloaded = __recv_bin.__tsa_unload();
            });

            for attr in attrs {
                attr.to_tokens(braced);
            }

            Ident::new("__recv_unloaded", method_span).to_tokens(braced);
            dot_token.to_tokens(braced);
            method.to_tokens(braced);
            turbofish.to_tokens(braced);
            paren_token.surround(braced, |parens| {
                for pair in args.pairs() {
                    write(parens).expr(pair.value());
                    pair.punct().to_tokens(parens);
                }
            });
            dot_await.to_tokens(braced);

            braced.extend(quote_spanned! {method_span=>
                .with_oof_builder(|| {
                    let fn_arg = FnArg::new(
                        VarType::var("self"),
                        __recv_ref_type,
                        __recv_type,
                        __recv_fn.call(),
                    );

                    let fn_context =
                        FnContext::new(__fn_is_async, __fn_name, vec![fn_arg]);

                    OofBuilder::new(fn_context.into())
                })
                #should_build
            });
        });
    }

    fn _call(self, call: &ExprCall) {
        let Oof {
            tokens,
            dot_await,
            should_build,
        } = self;

        let ExprCall {
            attrs,
            args,
            func,
            paren_token,
        } = call;

        Brace::default().surround(tokens, |braced| {
            braced.extend(quote!(
                use oofs::used_by_attribute::*;
            ));

            for attr in attrs {
                attr.to_tokens(braced);
            }

            write(braced).expr(func);
            paren_token.surround(braced, |parens| {
                for pair in args.pairs() {
                    write(parens).expr(pair.value());
                    pair.punct().to_tokens(parens);
                }
            });
            dot_await.to_tokens(braced);

            braced.extend(quote! {
                .with_oof_builder(|| {
                    OofBuilder::new(concat!(stringify!(#func), " failed").into())
                })
                #should_build
            });
        });
    }

    fn _await(self, expr_await: &ExprAwait) {
        let Oof {
            tokens,
            dot_await,
            should_build,
        } = self;

        let ExprAwait {
            attrs,
            base,
            dot_token,
            await_token,
        } = expr_await;

        for attr in attrs {
            attr.to_tokens(tokens);
        }

        oof(tokens).dot_await(dot_token, await_token).expr(base);

        should_build.to_tokens(tokens);

        dot_await.to_tokens(tokens);
    }

    fn _try(self, expr_try: &ExprTry) {
        let Oof {
            tokens,
            dot_await,
            should_build,
        } = self;

        let ExprTry {
            attrs,
            expr,
            question_token,
        } = expr_try;

        Brace::default().surround(tokens, |braced| {
            braced.extend(quote!(
                use oofs::used_by_attribute::*;
            ));

            for attr in attrs {
                attr.to_tokens(braced);
            }

            oof(braced).should_build().expr(expr);

            question_token.to_tokens(braced);
            dot_await.to_tokens(braced);

            braced.extend(quote! {
                .with_oof_builder(|| {
                    OofBuilder::new(concat!(stringify!(#expr), " failed").into())
                })
                #should_build
            });
        });
    }

    fn _other(self, expr: &Expr) {
        let Oof {
            tokens,
            dot_await,
            should_build,
            ..
        } = self;

        Brace::default().surround(tokens, |braced| {
            braced.extend(quote!(
                use oofs::used_by_attribute::*;
            ));

            write(braced).expr(expr);
            dot_await.to_tokens(braced);

            braced.extend(quote! {
                .with_oof_builder(|| {
                    OofBuilder::new(concat!(stringify!(#expr), " failed").into())
                })
                #should_build
            });
        });
    }
}
