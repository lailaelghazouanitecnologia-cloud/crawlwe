//! ZAD Parser - Parses tokens into AST

use crate::zad::ast::*;
use crate::zad::lexer::{Token, TokenInfo};
use crate::zad::{ZadError, Result};

pub struct Parser {
    tokens: Vec<TokenInfo>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<TokenInfo>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<Program> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            self.skip_newlines();
            if self.is_at_end() {
                break;
            }
            statements.push(self.parse_statement()?);
        }

        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        match self.peek() {
            Token::Template => self.parse_template(),
            Token::Import => self.parse_import(),
            Token::Const => self.parse_const(),
            Token::At => self.parse_directive_stmt(),
            _ => {
                Err(ZadError::ParserError {
                    line: self.current_line(),
                    message: format!("Unexpected token: {:?}", self.peek()),
                })
            }
        }
    }

    fn parse_template(&mut self) -> Result<Statement> {
        self.expect(Token::Template)?;

        let name = self.expect_ident()?;
        self.expect(Token::LParen)?;

        let params = self.parse_params()?;

        self.expect(Token::RParen)?;
        self.expect(Token::LBrace)?;

        let body = self.parse_template_body()?;

        self.expect(Token::RBrace)?;

        Ok(Statement::Template(TemplateDef {
            name,
            params,
            body,
            doc: None,
        }))
    }

    fn parse_params(&mut self) -> Result<Vec<Param>> {
        let mut params = Vec::new();

        while !self.check(&Token::RParen) {
            let name = self.expect_ident()?;
            self.expect(Token::Colon)?;
            let typ = self.parse_type()?;

            let default = if self.check(&Token::Eq) {
                self.advance();
                Some(self.parse_expr()?)
            } else {
                None
            };

            params.push(Param { name, typ, default });

            if !self.check(&Token::RParen) {
                self.expect(Token::Comma)?;
            }
        }

        Ok(params)
    }

    fn parse_type(&mut self) -> Result<Type> {
        // Handle array type: []Type
        if self.check(&Token::LBracket) {
            self.advance();
            self.expect(Token::RBracket)?;
            let inner = self.parse_type()?;
            return Ok(Type::Array(Box::new(inner)));
        }

        // Handle optional type: ?Type
        if self.check(&Token::Question) {
            self.advance();
            let inner = self.parse_type()?;
            return Ok(Type::Optional(Box::new(inner)));
        }

        // Basic types
        let name = self.expect_ident()?;
        Ok(Type::from_str(&name))
    }

    fn parse_template_body(&mut self) -> Result<TemplateBody> {
        let mut elements = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            elements.push(self.parse_template_element()?);
        }

        Ok(TemplateBody { elements })
    }

    fn parse_template_element(&mut self) -> Result<TemplateElement> {
        self.skip_newlines();

        match self.peek() {
            Token::For => self.parse_for_loop(),
            Token::If => self.parse_if_block(),
            Token::Switch => self.parse_switch_block(),
            Token::Let => self.parse_let_binding(),
            Token::At => self.parse_directive(),
            Token::LBrace => {
                // Interpolation: {expr}
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(Token::RBrace)?;
                Ok(TemplateElement::Interpolation(expr))
            }
            Token::String(s) => {
                let text = s.clone();
                self.advance();
                Ok(TemplateElement::Text(text))
            }
            Token::Ident(_) => {
                // Could be text or identifier
                let ident = self.expect_ident()?;
                Ok(TemplateElement::Text(ident))
            }
            Token::Newline => {
                self.advance();
                Ok(TemplateElement::Text("\n".to_string()))
            }
            _ => {
                // Treat as literal text
                let tok = self.peek().clone();
                self.advance();
                Ok(TemplateElement::Text(format!("{:?}", tok)))
            }
        }
    }

    fn parse_for_loop(&mut self) -> Result<TemplateElement> {
        self.expect(Token::For)?;
        self.expect(Token::LParen)?;

        let iterable = self.parse_expr()?;

        self.expect(Token::RParen)?;
        self.expect(Token::Pipe)?;

        let binding = self.expect_ident()?;

        let index_binding = if self.check(&Token::Comma) {
            self.advance();
            Some(self.expect_ident()?)
        } else {
            None
        };

        self.expect(Token::Pipe)?;
        self.expect(Token::LBrace)?;

        let body = self.parse_template_body()?;

        self.expect(Token::RBrace)?;

        Ok(TemplateElement::For(ForLoop {
            iterable,
            binding,
            index_binding,
            body,
            separator: None,
        }))
    }

    fn parse_if_block(&mut self) -> Result<TemplateElement> {
        self.expect(Token::If)?;
        self.expect(Token::LParen)?;

        let condition = self.parse_expr()?;

        self.expect(Token::RParen)?;
        self.expect(Token::LBrace)?;

        let then_body = self.parse_template_body()?;

        self.expect(Token::RBrace)?;

        let mut else_if = Vec::new();
        let mut else_body = None;

        while self.check(&Token::Else) {
            self.advance();

            if self.check(&Token::If) {
                self.advance();
                self.expect(Token::LParen)?;
                let cond = self.parse_expr()?;
                self.expect(Token::RParen)?;
                self.expect(Token::LBrace)?;
                let body = self.parse_template_body()?;
                self.expect(Token::RBrace)?;
                else_if.push((cond, body));
            } else {
                self.expect(Token::LBrace)?;
                else_body = Some(self.parse_template_body()?);
                self.expect(Token::RBrace)?;
                break;
            }
        }

        Ok(TemplateElement::If(IfBlock {
            condition,
            then_body,
            else_if,
            else_body,
        }))
    }

    fn parse_switch_block(&mut self) -> Result<TemplateElement> {
        self.expect(Token::Switch)?;
        self.expect(Token::LParen)?;

        let value = self.parse_expr()?;

        self.expect(Token::RParen)?;
        self.expect(Token::LBrace)?;

        let mut cases = Vec::new();
        let mut default = None;

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            self.skip_newlines();

            if self.check(&Token::Else) {
                self.advance();
                self.expect(Token::Arrow)?;
                self.expect(Token::LBrace)?;
                default = Some(self.parse_template_body()?);
                self.expect(Token::RBrace)?;
                break;
            }

            // Parse case pattern
            let pattern = self.parse_pattern()?;

            self.expect(Token::Arrow)?;

            // Optional binding
            let binding = if self.check(&Token::Pipe) {
                self.advance();
                let b = self.expect_ident()?;
                self.expect(Token::Pipe)?;
                Some(b)
            } else {
                None
            };

            self.expect(Token::LBrace)?;
            let body = self.parse_template_body()?;
            self.expect(Token::RBrace)?;

            // Optional comma
            if self.check(&Token::Comma) {
                self.advance();
            }

            cases.push(SwitchCase { pattern, binding, body });
        }

        self.expect(Token::RBrace)?;

        Ok(TemplateElement::Switch(SwitchBlock { value, cases, default }))
    }

    fn parse_pattern(&mut self) -> Result<Pattern> {
        if self.check(&Token::Dot) {
            self.advance();
            let variant = self.expect_ident()?;
            Ok(Pattern::Variant(variant))
        } else {
            let lit = self.parse_literal()?;
            Ok(Pattern::Literal(lit))
        }
    }

    fn parse_let_binding(&mut self) -> Result<TemplateElement> {
        self.expect(Token::Let)?;
        let name = self.expect_ident()?;
        self.expect(Token::Eq)?;
        let value = self.parse_expr()?;
        self.expect(Token::Semi)?;

        Ok(TemplateElement::Let(LetBinding { name, value }))
    }

    fn parse_directive(&mut self) -> Result<TemplateElement> {
        self.expect(Token::At)?;
        let name = self.expect_ident()?;

        let args = if self.check(&Token::LParen) {
            self.advance();
            let args = self.parse_args()?;
            self.expect(Token::RParen)?;
            args
        } else {
            Vec::new()
        };

        // Handle @use specially
        if name == "use" && !args.is_empty() {
            if let Expr::Var(template_name) = &args[0] {
                return Ok(TemplateElement::Use(UseStmt {
                    template: template_name.clone(),
                    args: args.into_iter().skip(1).collect(),
                }));
            }
        }

        Ok(TemplateElement::Directive(Directive { name, args }))
    }

    fn parse_directive_stmt(&mut self) -> Result<Statement> {
        self.expect(Token::At)?;
        let name = self.expect_ident()?;

        let args = if self.check(&Token::LParen) {
            self.advance();
            let args = self.parse_args()?;
            self.expect(Token::RParen)?;
            args
        } else {
            Vec::new()
        };

        // Optional semicolon
        if self.check(&Token::Semi) {
            self.advance();
        }

        if name == "emit" {
            // @emit template(args)
            if let Some(Expr::Call(template_expr, call_args)) = args.into_iter().next() {
                if let Expr::Var(template_name) = *template_expr {
                    return Ok(Statement::Emit(EmitStmt {
                        template: template_name,
                        args: call_args,
                    }));
                }
            }
            return Err(ZadError::ParserError {
                line: self.current_line(),
                message: "@emit requires a template call".into(),
            });
        }

        // Generic directive at statement level
        Ok(Statement::Raw(format!("@{}({:?})", name, args)))
    }

    fn parse_import(&mut self) -> Result<Statement> {
        self.expect(Token::Import)?;

        // import "path" or import { items } from "path"
        if self.check(&Token::LBrace) {
            self.advance();
            let mut items = Vec::new();
            while !self.check(&Token::RBrace) {
                items.push(self.expect_ident()?);
                if !self.check(&Token::RBrace) {
                    self.expect(Token::Comma)?;
                }
            }
            self.expect(Token::RBrace)?;
            self.expect(Token::From)?;

            let path = self.expect_string()?;
            self.expect(Token::Semi)?;

            Ok(Statement::Import(ImportStmt {
                path,
                items: Some(items),
            }))
        } else {
            let path = self.expect_string()?;
            self.expect(Token::Semi)?;

            Ok(Statement::Import(ImportStmt {
                path,
                items: None,
            }))
        }
    }

    fn parse_const(&mut self) -> Result<Statement> {
        self.expect(Token::Const)?;
        let name = self.expect_ident()?;
        self.expect(Token::Eq)?;
        let value = self.parse_expr()?;
        self.expect(Token::Semi)?;

        Ok(Statement::Const(ConstDef { name, value }))
    }

    fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_ternary()
    }

    fn parse_ternary(&mut self) -> Result<Expr> {
        let mut expr = self.parse_or()?;

        if self.check(&Token::Question) {
            self.advance();
            let then_expr = self.parse_expr()?;
            self.expect(Token::Colon)?;
            let else_expr = self.parse_expr()?;
            expr = Expr::Ternary(Box::new(expr), Box::new(then_expr), Box::new(else_expr));
        }

        Ok(expr)
    }

    fn parse_or(&mut self) -> Result<Expr> {
        let mut left = self.parse_and()?;

        while self.check(&Token::Or) {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::Binary(Box::new(left), BinOp::Or, Box::new(right));
        }

        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr> {
        let mut left = self.parse_equality()?;

        while self.check(&Token::And) {
            self.advance();
            let right = self.parse_equality()?;
            left = Expr::Binary(Box::new(left), BinOp::And, Box::new(right));
        }

        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr> {
        let mut left = self.parse_comparison()?;

        loop {
            let op = match self.peek() {
                Token::EqEq => BinOp::Eq,
                Token::Ne => BinOp::Ne,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr> {
        let mut left = self.parse_concat()?;

        loop {
            let op = match self.peek() {
                Token::Lt => BinOp::Lt,
                Token::Le => BinOp::Le,
                Token::Gt => BinOp::Gt,
                Token::Ge => BinOp::Ge,
                _ => break,
            };
            self.advance();
            let right = self.parse_concat()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }

        Ok(left)
    }

    fn parse_concat(&mut self) -> Result<Expr> {
        let mut left = self.parse_additive()?;

        while self.check(&Token::Concat) {
            self.advance();
            let right = self.parse_additive()?;
            left = Expr::Binary(Box::new(left), BinOp::Concat, Box::new(right));
        }

        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr> {
        let mut left = self.parse_multiplicative()?;

        loop {
            let op = match self.peek() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }

        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr> {
        let mut left = self.parse_unary()?;

        loop {
            let op = match self.peek() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        match self.peek() {
            Token::Not => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary(UnaryOp::Not, Box::new(expr)))
            }
            Token::Minus => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary(UnaryOp::Neg, Box::new(expr)))
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr> {
        let mut expr = self.parse_primary()?;

        loop {
            match self.peek() {
                Token::Dot => {
                    self.advance();
                    let field = self.expect_ident()?;
                    expr = Expr::Field(Box::new(expr), field);
                }
                Token::LBracket => {
                    self.advance();
                    let index = self.parse_expr()?;
                    self.expect(Token::RBracket)?;
                    expr = Expr::Index(Box::new(expr), Box::new(index));
                }
                Token::LParen => {
                    self.advance();
                    let args = self.parse_args()?;
                    self.expect(Token::RParen)?;
                    expr = Expr::Call(Box::new(expr), args);
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        match self.peek() {
            Token::True => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(true)))
            }
            Token::False => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(false)))
            }
            Token::Null => {
                self.advance();
                Ok(Expr::Literal(Literal::Null))
            }
            Token::Int(i) => {
                let val = *i;
                self.advance();
                Ok(Expr::Literal(Literal::Int(val)))
            }
            Token::Float(f) => {
                let val = *f;
                self.advance();
                Ok(Expr::Literal(Literal::Float(val)))
            }
            Token::String(s) => {
                let val = s.clone();
                self.advance();
                Ok(Expr::Literal(Literal::String(val)))
            }
            Token::Ident(name) => {
                let name = name.clone();
                self.advance();
                Ok(Expr::Var(name))
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            Token::LBracket => {
                self.advance();
                let elements = self.parse_array_elements()?;
                self.expect(Token::RBracket)?;
                Ok(Expr::Array(elements))
            }
            Token::LBrace => {
                self.advance();
                let fields = self.parse_object_fields()?;
                self.expect(Token::RBrace)?;
                Ok(Expr::Object(fields))
            }
            Token::Dot => {
                // .variant shorthand
                self.advance();
                let variant = self.expect_ident()?;
                Ok(Expr::Var(format!(".{}", variant)))
            }
            _ => Err(ZadError::ParserError {
                line: self.current_line(),
                message: format!("Expected expression, got {:?}", self.peek()),
            }),
        }
    }

    fn parse_literal(&mut self) -> Result<Literal> {
        match self.peek() {
            Token::True => {
                self.advance();
                Ok(Literal::Bool(true))
            }
            Token::False => {
                self.advance();
                Ok(Literal::Bool(false))
            }
            Token::Null => {
                self.advance();
                Ok(Literal::Null)
            }
            Token::Int(i) => {
                let val = *i;
                self.advance();
                Ok(Literal::Int(val))
            }
            Token::Float(f) => {
                let val = *f;
                self.advance();
                Ok(Literal::Float(val))
            }
            Token::String(s) => {
                let val = s.clone();
                self.advance();
                Ok(Literal::String(val))
            }
            _ => Err(ZadError::ParserError {
                line: self.current_line(),
                message: format!("Expected literal, got {:?}", self.peek()),
            }),
        }
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>> {
        let mut args = Vec::new();

        while !self.check(&Token::RParen) {
            args.push(self.parse_expr()?);
            if !self.check(&Token::RParen) {
                self.expect(Token::Comma)?;
            }
        }

        Ok(args)
    }

    fn parse_array_elements(&mut self) -> Result<Vec<Expr>> {
        let mut elements = Vec::new();

        while !self.check(&Token::RBracket) {
            self.skip_newlines();
            if self.check(&Token::RBracket) {
                break;
            }
            elements.push(self.parse_expr()?);
            self.skip_newlines();
            if !self.check(&Token::RBracket) {
                self.expect(Token::Comma)?;
            }
        }

        Ok(elements)
    }

    fn parse_object_fields(&mut self) -> Result<Vec<(String, Expr)>> {
        let mut fields = Vec::new();

        while !self.check(&Token::RBrace) {
            self.skip_newlines();
            if self.check(&Token::RBrace) {
                break;
            }

            let key = self.expect_ident()?;
            self.expect(Token::Colon)?;
            let value = self.parse_expr()?;

            fields.push((key, value));

            self.skip_newlines();
            if !self.check(&Token::RBrace) {
                self.expect(Token::Comma)?;
            }
        }

        Ok(fields)
    }

    // Helper methods

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).map(|t| &t.token).unwrap_or(&Token::Eof)
    }

    fn check(&self, token: &Token) -> bool {
        std::mem::discriminant(self.peek()) == std::mem::discriminant(token)
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.pos += 1;
        }
        self.tokens.get(self.pos - 1).map(|t| &t.token).unwrap_or(&Token::Eof)
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }

    fn current_line(&self) -> usize {
        self.tokens.get(self.pos).map(|t| t.line).unwrap_or(0)
    }

    fn skip_newlines(&mut self) {
        while self.check(&Token::Newline) {
            self.advance();
        }
    }

    fn expect(&mut self, token: Token) -> Result<()> {
        if self.check(&token) {
            self.advance();
            Ok(())
        } else {
            Err(ZadError::ParserError {
                line: self.current_line(),
                message: format!("Expected {:?}, got {:?}", token, self.peek()),
            })
        }
    }

    fn expect_ident(&mut self) -> Result<String> {
        match self.peek() {
            Token::Ident(name) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            _ => Err(ZadError::ParserError {
                line: self.current_line(),
                message: format!("Expected identifier, got {:?}", self.peek()),
            }),
        }
    }

    fn expect_string(&mut self) -> Result<String> {
        match self.peek() {
            Token::String(s) => {
                let s = s.clone();
                self.advance();
                Ok(s)
            }
            _ => Err(ZadError::ParserError {
                line: self.current_line(),
                message: format!("Expected string, got {:?}", self.peek()),
            }),
        }
    }
}
