use super::context::context;
use quote::ToTokens;
use syn::{token::Semi, *};

pub fn write<'a>(tokens: &'a mut proc_macro2::TokenStream) -> Writer<'a> {
    Writer::new(tokens)
}

pub struct Writer<'a> {
    tokens: &'a mut proc_macro2::TokenStream,
}

impl<'a> Writer<'a> {
    fn new(tokens: &'a mut proc_macro2::TokenStream) -> Self {
        Self { tokens }
    }

    pub fn block(self, block: &Block) {
        let Self { tokens, .. } = self;

        block.brace_token.surround(tokens, |braced| {
            write(braced).stmts(&block.stmts);
        });
    }

    fn stmts(self, stmts: &Vec<Stmt>) {
        let Self { tokens, .. } = self;
        for stmt in stmts {
            match stmt {
                Stmt::Local(local) => write(tokens).local(local),
                Stmt::Item(item) => write(tokens).item(item),
                Stmt::Semi(expr, semi) => write(tokens).semi(expr, semi),
                Stmt::Expr(expr) => write(tokens).expr(expr),
            }
        }
    }

    fn local(self, local: &Local) {
        let Self { tokens, .. } = self;
        let Local {
            attrs,
            let_token,
            pat,
            init,
            semi_token,
        } = local;

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        let_token.to_tokens(tokens);
        pat.to_tokens(tokens);

        if let Some((eq, expr)) = init {
            eq.to_tokens(tokens);
            write(tokens).expr(expr);
        }

        semi_token.to_tokens(tokens);
    }

    fn item(self, item: &Item) {
        item.to_tokens(self.tokens);
    }

    fn semi(self, expr: &Expr, semi: &Semi) {
        write(self.tokens).expr(expr);
        semi.to_tokens(self.tokens);
    }

    pub fn expr(self, expr: &Expr) {
        match expr {
            Expr::Try(_try) => self._try(_try), // main case for handling results
            Expr::Return(_return) => self._return(_return),
            // Rest of the cases look for inner expr and recurse `write(tokens).expr(expr)`.
            Expr::Array(_array) => self._array(_array),
            Expr::Assign(_assign) => self._assign(_assign),
            Expr::AssignOp(_assign_op) => self._assign_op(_assign_op),
            Expr::Await(_await) => self._await(_await),
            Expr::Binary(_binary) => self._binary(_binary),
            Expr::Block(_block) => self._block(_block),
            Expr::Box(_box) => self._box(_box),
            Expr::Break(_break) => self._break(_break),
            Expr::Call(_call) => self._call(_call),
            Expr::Cast(_cast) => self._cast(_cast),
            Expr::Field(_field) => self._field(_field),
            Expr::ForLoop(_for_loop) => self._for_loop(_for_loop),
            Expr::Group(_group) => self._group(_group),
            Expr::If(_if) => self._if(_if),
            Expr::Index(_index) => self._index(_index),
            Expr::Loop(_loop) => self._loop(_loop),
            Expr::Match(_match) => self._match(_match),
            Expr::MethodCall(_method_call) => self._method_call(_method_call),
            Expr::Paren(_paren) => self._paren(_paren),
            Expr::Range(_range) => self._range(_range),
            Expr::Reference(_reference) => self._reference(_reference),
            Expr::Repeat(_repeat) => self._repeat(_repeat),
            Expr::Struct(_struct) => self._struct(_struct),
            Expr::TryBlock(_try_block) => self._try_block(_try_block),
            Expr::Tuple(_tuple) => self._tuple(_tuple),
            Expr::Type(_type) => self._type(_type),
            Expr::Unary(_unary) => self._unary(_unary),
            Expr::Unsafe(_unsafe) => self._unsafe(_unsafe),
            Expr::While(_while) => self._while(_while),
            Expr::Yield(_yield) => self._yield(_yield),
            // unhandled cases:
            // async blocks, closures, continue, literals, macros, path, verbatim
            expr => expr.to_tokens(self.tokens),
        }
    }

    fn _try(self, _try: &ExprTry) {
        let Self { tokens, .. } = self;
        let ExprTry {
            attrs,
            expr,
            question_token,
        } = _try;

        for attr in attrs {
            attr.to_tokens(tokens);
        }

        context(tokens).expr(expr);

        question_token.to_tokens(tokens);
    }

    fn _return(self, _return: &ExprReturn) {
        let Self { tokens, .. } = self;
        let ExprReturn {
            attrs,
            return_token,
            expr,
        } = _return;

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        return_token.to_tokens(tokens);

        if let Some(expr) = expr {
            write(tokens).expr(expr);
        }
    }

    fn _array(self, _array: &ExprArray) {
        let Self { tokens, .. } = self;
        let ExprArray {
            attrs,
            bracket_token,
            elems,
        } = _array;

        for attr in attrs {
            attr.to_tokens(tokens);
        }

        bracket_token.surround(tokens, |bracket| {
            for pair in elems.pairs() {
                write(bracket).expr(pair.value());
                pair.punct().to_tokens(bracket);
            }
        });
    }

    fn _assign(self, _assign: &ExprAssign) {
        let Self { tokens, .. } = self;
        let ExprAssign {
            attrs,
            left,
            eq_token,
            right,
        } = _assign;

        for attr in attrs {
            attr.to_tokens(tokens);
        }

        write(tokens).expr(left);
        eq_token.to_tokens(tokens);
        write(tokens).expr(right);
    }

    fn _assign_op(self, _assign_op: &ExprAssignOp) {
        let Self { tokens, .. } = self;
        let ExprAssignOp {
            attrs,
            left,
            op,
            right,
        } = _assign_op;

        for attr in attrs {
            attr.to_tokens(tokens);
        }

        write(tokens).expr(left);
        op.to_tokens(tokens);
        write(tokens).expr(right);
    }

    fn _await(self, _await: &ExprAwait) {
        let Self { tokens, .. } = self;
        let ExprAwait {
            attrs,
            base,
            dot_token,
            await_token,
        } = _await;

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        write(tokens).expr(base);
        dot_token.to_tokens(tokens);
        await_token.to_tokens(tokens);
    }

    fn _binary(self, _binary: &ExprBinary) {
        let Self { tokens, .. } = self;
        let ExprBinary {
            attrs,
            left,
            op,
            right,
        } = _binary;

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        write(tokens).expr(left);
        op.to_tokens(tokens);
        write(tokens).expr(right);
    }

    fn _block(self, _block: &ExprBlock) {
        let Self { tokens, .. } = self;
        let ExprBlock {
            attrs,
            label,
            block,
        } = _block;

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        label.to_tokens(tokens);
        write(tokens).block(block);
    }

    fn _box(self, _box: &ExprBox) {
        let Self { tokens, .. } = self;
        let ExprBox {
            attrs,
            box_token,
            expr,
        } = _box;

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        box_token.to_tokens(tokens);
        write(tokens).expr(expr);
    }

    fn _break(self, _break: &ExprBreak) {
        let Self { tokens, .. } = self;
        let ExprBreak {
            attrs,
            break_token,
            label,
            expr,
        } = _break;

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        break_token.to_tokens(tokens);
        label.to_tokens(tokens);
        if let Some(expr) = expr {
            write(tokens).expr(expr);
        }
    }

    fn _call(self, _call: &ExprCall) {
        let Self { tokens, .. } = self;
        let ExprCall {
            attrs,
            func,
            paren_token,
            args,
        } = _call;

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        write(tokens).expr(func);
        paren_token.surround(tokens, |parens| {
            for pair in args.pairs() {
                write(parens).expr(pair.value());
                pair.punct().to_tokens(parens);
            }
        });
    }

    fn _cast(self, _cast: &ExprCast) {
        let Self { tokens, .. } = self;
        let ExprCast {
            attrs,
            expr,
            as_token,
            ty,
        } = _cast;

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        write(tokens).expr(expr);
        as_token.to_tokens(tokens);
        ty.to_tokens(tokens);
    }

    fn _field(self, _field: &ExprField) {
        let Self { tokens, .. } = self;
        let ExprField {
            attrs,
            base,
            dot_token,
            member,
        } = _field;

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        write(tokens).expr(base);
        dot_token.to_tokens(tokens);
        member.to_tokens(tokens);
    }

    fn _for_loop(self, _for_loop: &ExprForLoop) {
        let Self { tokens, .. } = self;
        let ExprForLoop {
            attrs,
            label,
            for_token,
            pat,
            in_token,
            expr,
            body,
        } = _for_loop;

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        label.to_tokens(tokens);
        for_token.to_tokens(tokens);
        pat.to_tokens(tokens);
        in_token.to_tokens(tokens);
        write(tokens).expr(expr);
        write(tokens).block(body);
    }

    fn _group(self, _group: &ExprGroup) {
        let Self { tokens, .. } = self;
        let ExprGroup {
            attrs,
            group_token,
            expr,
        } = _group;

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        group_token.surround(tokens, |grouped| {
            write(grouped).expr(expr);
        });
    }

    fn _if(self, _if: &ExprIf) {
        let Self { tokens, .. } = self;
        let ExprIf {
            attrs,
            if_token,
            cond,
            then_branch,
            else_branch,
        } = _if;

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        if_token.to_tokens(tokens);
        write(tokens).expr(cond);
        write(tokens).block(then_branch);
        if let Some((else_token, expr)) = else_branch {
            else_token.to_tokens(tokens);
            write(tokens).expr(expr);
        }
    }

    fn _index(self, _index: &ExprIndex) {
        let Self { tokens, .. } = self;
        let ExprIndex {
            attrs,
            expr,
            bracket_token,
            index,
        } = _index;

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        write(tokens).expr(expr);
        bracket_token.surround(tokens, |bracket| {
            write(bracket).expr(index);
        })
    }

    fn _let(self, _let: &ExprLet) {
        let Self { tokens, .. } = self;
        let ExprLet {
            attrs,
            let_token,
            pat,
            eq_token,
            expr,
        } = _let;

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        let_token.to_tokens(tokens);
        pat.to_tokens(tokens);
        eq_token.to_tokens(tokens);
        write(tokens).expr(expr);
    }

    fn _loop(self, _loop: &ExprLoop) {
        let Self { tokens, .. } = self;
        let ExprLoop {
            attrs,
            label,
            loop_token,
            body,
        } = _loop;

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        label.to_tokens(tokens);
        loop_token.to_tokens(tokens);
        write(tokens).block(body);
    }

    fn _match(self, _match: &ExprMatch) {
        let Self { tokens, .. } = self;
        let ExprMatch {
            attrs,
            match_token,
            expr,
            brace_token,
            arms,
        } = _match;

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        match_token.to_tokens(tokens);
        write(tokens).expr(expr);
        brace_token.surround(tokens, |braces| {
            for arm in arms {
                let Arm {
                    attrs,
                    pat,
                    guard,
                    fat_arrow_token,
                    body,
                    comma,
                } = arm;

                for attr in attrs {
                    attr.to_tokens(braces);
                }
                pat.to_tokens(braces);
                if let Some((if_token, expr)) = guard {
                    if_token.to_tokens(braces);
                    write(braces).expr(expr);
                }
                fat_arrow_token.to_tokens(braces);
                write(braces).expr(body);
                comma.to_tokens(braces);
            }
        });
    }

    fn _method_call(self, _method_call: &ExprMethodCall) {
        let Self { tokens, .. } = self;
        let ExprMethodCall {
            attrs,
            receiver,
            dot_token,
            method,
            turbofish,
            paren_token,
            args,
        } = _method_call;

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        write(tokens).expr(receiver);
        dot_token.to_tokens(tokens);
        method.to_tokens(tokens);
        turbofish.to_tokens(tokens);
        paren_token.surround(tokens, |parens| {
            for pair in args.pairs() {
                write(parens).expr(pair.value());
                pair.punct().to_tokens(parens);
            }
        });
    }

    fn _paren(self, _paren: &ExprParen) {
        let Self { tokens, .. } = self;
        let ExprParen {
            attrs,
            paren_token,
            expr,
        } = _paren;

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        paren_token.surround(tokens, |parens| {
            write(parens).expr(expr);
        });
    }

    fn _range(self, _range: &ExprRange) {
        let Self { tokens, .. } = self;
        let ExprRange {
            attrs,
            from,
            limits,
            to,
        } = _range;

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        if let Some(from) = from {
            write(tokens).expr(from);
        }
        limits.to_tokens(tokens);
        if let Some(to) = to {
            write(tokens).expr(to);
        }
    }

    fn _reference(self, _reference: &ExprReference) {
        let Self { tokens, .. } = self;
        let ExprReference {
            attrs,
            and_token,
            mutability,
            expr,
            ..
        } = _reference;

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        and_token.to_tokens(tokens);
        mutability.to_tokens(tokens);
        write(tokens).expr(expr);
    }

    fn _repeat(self, _repeat: &ExprRepeat) {
        let Self { tokens, .. } = self;
        let ExprRepeat {
            attrs,
            bracket_token,
            expr,
            semi_token,
            len,
        } = _repeat;

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        bracket_token.surround(tokens, |bracket| {
            write(bracket).expr(expr);
            semi_token.to_tokens(bracket);
            write(bracket).expr(len);
        });
    }

    fn _struct(self, _struct: &ExprStruct) {
        let Self { tokens, .. } = self;
        let ExprStruct {
            attrs,
            path,
            brace_token,
            fields,
            dot2_token,
            rest,
        } = _struct;

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        path.to_tokens(tokens);
        brace_token.surround(tokens, |braced| {
            for pair in fields.pairs() {
                let FieldValue {
                    attrs,
                    member,
                    colon_token,
                    expr,
                } = pair.value();

                for attr in attrs {
                    attr.to_tokens(braced);
                }
                member.to_tokens(braced);
                colon_token.to_tokens(braced);
                write(braced).expr(expr);

                pair.punct().to_tokens(braced);
            }
            dot2_token.to_tokens(braced);
            if let Some(rest) = rest {
                write(braced).expr(rest);
            }
        });
    }

    fn _try_block(self, _try_block: &ExprTryBlock) {
        let Self { tokens, .. } = self;
        let ExprTryBlock {
            attrs,
            try_token,
            block,
        } = _try_block;

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        try_token.to_tokens(tokens);
        write(tokens).block(block);
    }

    fn _tuple(self, _tuple: &ExprTuple) {
        let Self { tokens, .. } = self;
        let ExprTuple {
            attrs,
            paren_token,
            elems,
        } = _tuple;

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        paren_token.surround(tokens, |parens| {
            for elem in elems.pairs() {
                write(parens).expr(elem.value());
                elem.punct().to_tokens(parens);
            }
        });
    }

    fn _type(self, _type: &ExprType) {
        let Self { tokens, .. } = self;
        let ExprType {
            attrs,
            expr,
            colon_token,
            ty,
        } = _type;

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        write(tokens).expr(expr);
        colon_token.to_tokens(tokens);
        ty.to_tokens(tokens);
    }

    fn _unary(self, _unary: &ExprUnary) {
        let Self { tokens, .. } = self;
        let ExprUnary { attrs, op, expr } = _unary;

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        op.to_tokens(tokens);
        write(tokens).expr(expr);
    }

    fn _unsafe(self, _unsafe: &ExprUnsafe) {
        let Self { tokens, .. } = self;
        let ExprUnsafe {
            attrs,
            unsafe_token,
            block,
        } = _unsafe;

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        unsafe_token.to_tokens(tokens);
        write(tokens).block(block);
    }

    fn _while(self, _while: &ExprWhile) {
        let Self { tokens, .. } = self;
        let ExprWhile {
            attrs,
            label,
            while_token,
            cond,
            body,
        } = _while;

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        label.to_tokens(tokens);
        while_token.to_tokens(tokens);
        write(tokens).expr(cond);
        write(tokens).block(body);
    }

    fn _yield(self, _yield: &ExprYield) {
        let Self { tokens, .. } = self;
        let ExprYield {
            attrs,
            yield_token,
            expr,
        } = _yield;

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        yield_token.to_tokens(tokens);
        if let Some(expr) = expr {
            write(tokens).expr(expr);
        }
    }
}

fn is_generic_ok(expr: &Expr) -> bool {
    if let Expr::Call(call) = expr {
        if let Expr::Path(path) = call.func.as_ref() {
            return path
                .path
                .segments
                .last()
                .map(|s| s.ident == "Ok" && matches!(s.arguments, PathArguments::None))
                .unwrap_or(false);
        }
    }

    false
}
