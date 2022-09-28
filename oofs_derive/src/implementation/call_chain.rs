use super::{write::write, OOF_METHODS};
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::{
    token::{Await, Brace, Dot, Semi},
    Expr, ExprAwait, ExprCall, ExprMethodCall, ExprTry,
};

fn build_call_chain(model: &mut Vec<CallModel>, expr: &Expr, is_async: bool) {
    use Expr::*;

    match expr {
        MethodCall(call) => build_method_call(model, call, is_async),
        Call(call) => build_call(model, call, is_async),
        Await(ExprAwait { attrs, base, .. }) => build_call_chain(model, base, true),
        expr => {}
    }
}

fn build_method_call(model: &mut Vec<CallModel>, expr: &ExprMethodCall, is_async: bool) {}
fn build_call(model: &mut Vec<CallModel>, expr: &ExprCall, is_async: bool) {}

enum CallModel {
    Method { args: Vec<String>, is_async: bool },
    Fn { args: Vec<String>, is_async: bool },
    Other(Expr),
}

impl ToTokens for CallModel {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {}
}
