use std::fmt;

use rowan::{GreenNode, GreenNodeBuilder};

use crate::syntax::{LexedStr, LexedStrIter, SyntaxKind, TokenIdx};

#[derive(Debug)]
pub struct Parsed {
    pub green_node: GreenNode,
    pub errors: Vec<ParserError>,
    pub unexpected_eof: bool,
}

#[derive(Debug)]
pub struct ParserError {
    pub message: String,
    pub token_idx: TokenIdx,
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

struct Parser<'a> {
    builder: GreenNodeBuilder<'static>,
    tokens: LexedStrIter<'a>,
    errors: Vec<ParserError>,
    unexpected_eof: bool,
}

pub fn parse(lexed: &LexedStr<'_>) -> Parsed {
    Parser {
        builder: GreenNodeBuilder::new(),
        tokens: lexed.iter(),
        errors: Vec::new(),
        unexpected_eof: false,
    }
    .parse()
}

impl Parser<'_> {
    fn skip_noise(&mut self) {
        while let Some((
            SyntaxKind::Whitespace | SyntaxKind::LineComment | SyntaxKind::BlockComment,
            _,
        )) = self.tokens.peek()
        {
            self.bump();
        }
    }

    fn bump(&mut self) {
        let (token, text) = self.tokens.next().unwrap();
        self.builder.token(token.into(), text);
    }

    fn current(&mut self) -> Option<SyntaxKind> {
        self.skip_noise();
        self.tokens.peek().map(|(token, _)| token)
    }

    fn parse(mut self) -> Parsed {
        self.builder.start_node(SyntaxKind::Root.into());

        while let Some(token) = self.current() {
            match token {
                SyntaxKind::Functor => self.parse_rule_or_belief(),
                SyntaxKind::Bang => self.parse_initial_goal(),
                SyntaxKind::At | SyntaxKind::Plus | SyntaxKind::Minus => self.parse_plan(),
                _ => self.recover(
                    format!("unexpected token {:?}", token),
                    |t| t == SyntaxKind::Dot,
                    |_| false,
                ),
            }
        }

        self.builder.finish_node(); // root

        Parsed {
            green_node: self.builder.finish(),
            errors: self.errors,
            unexpected_eof: self.unexpected_eof,
        }
    }

    fn parse_rule_or_belief(&mut self) {
        let checkpoint = self.builder.checkpoint();
        self.parse_literal();

        if self.current() == Some(SyntaxKind::Define) {
            self.builder
                .start_node_at(checkpoint, SyntaxKind::Rule.into());
            self.bump();
            self.parse_term();
        } else {
            self.builder
                .start_node_at(checkpoint, SyntaxKind::Belief.into());
        }

        if self.current() == Some(SyntaxKind::Dot) {
            self.bump();
        } else {
            self.recover(
                "expected '.' after rule or belief",
                |t| t == SyntaxKind::Dot,
                |_| false,
            );
        }

        self.builder.finish_node();
    }

    fn parse_initial_goal(&mut self) {
        self.builder.start_node(SyntaxKind::InitialGoal.into());

        assert!(self.current() == Some(SyntaxKind::Bang));
        self.bump();

        match self.current() {
            Some(SyntaxKind::Functor) => self.parse_literal(),
            Some(token) => {
                self.recover(
                    format!("expected functor after '!', got {:?}", token),
                    |t| t == SyntaxKind::Dot,
                    |_| false,
                );
                self.builder.finish_node();
                return;
            }
            None => {
                self.unexpected_eof = true;
                self.builder.finish_node();
                return;
            }
        }

        match self.current() {
            Some(SyntaxKind::Dot) => self.bump(),
            Some(token) => self.recover(
                format!("expected '.' after initial goal, got {:?}", token),
                |t| t == SyntaxKind::Dot,
                |_| false,
            ),
            None => self.unexpected_eof = true,
        }

        self.builder.finish_node();
    }

    fn parse_plan(&mut self) {
        self.builder.start_node(SyntaxKind::Plan.into());

        while self.current() == Some(SyntaxKind::At) {
            self.builder.start_node(SyntaxKind::PlanAnnotation.into());
            self.bump();
            self.parse_literal();
            self.builder.finish_node();
        }

        match self.current() {
            Some(SyntaxKind::Plus | SyntaxKind::Minus) => self.bump(),
            _ => self.push_error("expected '+' or '-' for plan trigger"),
        }

        if let Some(SyntaxKind::Bang) = self.current() {
            self.bump();
        }

        self.parse_literal();

        if self.current() == Some(SyntaxKind::Colon) {
            self.bump();
            self.builder.start_node(SyntaxKind::PlanContext.into());
            self.parse_term();
            self.builder.finish_node();
        }

        if self.current() == Some(SyntaxKind::Arrow) {
            self.bump();
            self.builder.start_node(SyntaxKind::Body.into());
            loop {
                self.parse_formula();
                match self.current() {
                    Some(SyntaxKind::Semi) => self.bump(),
                    Some(SyntaxKind::Dot) => {
                        self.bump();
                        break;
                    }
                    Some(token) => self.recover(
                        format!("expected ';' or '.', got {:?}", token),
                        |_| false,
                        |t| t == SyntaxKind::Semi || t == SyntaxKind::Dot,
                    ),
                    None => {
                        self.unexpected_eof = true;
                        break;
                    }
                }
            }
            self.builder.finish_node();
        }

        self.builder.finish_node();
    }

    fn parse_formula(&mut self) {
        self.builder.start_node(SyntaxKind::Formula.into());
        match self.current() {
            Some(token) if token.formula_type().is_some() => self.bump(),
            Some(SyntaxKind::While | SyntaxKind::If | SyntaxKind::For) => todo!("control flow"),
            Some(_) => (),
            None => self.unexpected_eof = true,
        }
        self.parse_term();
        self.builder.finish_node();
    }

    fn parse_literal(&mut self) {
        self.builder.start_node(SyntaxKind::Literal.into());

        match self.current() {
            Some(SyntaxKind::Functor) => self.bump(),
            Some(token) => {
                self.recover(
                    format!("expected literal, got {:?}", token),
                    |_| false,
                    |t| t == SyntaxKind::Dot || t == SyntaxKind::Semi,
                );
                self.builder.finish_node();
                return;
            }
            None => self.unexpected_eof = true,
        }

        if self.current() == Some(SyntaxKind::OpenParen) {
            self.builder.start_node(SyntaxKind::LiteralTerms.into());
            self.bump();

            self.parse_term();
            while let Some(SyntaxKind::Comma) = self.current() {
                self.bump();
                self.parse_term();
            }

            match self.current() {
                Some(SyntaxKind::CloseParen) => self.bump(),
                Some(token) => {
                    self.recover(
                        format!("expected ')' to close literal, got {:?}", token),
                        |t| t == SyntaxKind::CloseParen,
                        |t| t == SyntaxKind::Dot || t == SyntaxKind::Semi,
                    );
                }
                None => self.unexpected_eof = true,
            }

            self.builder.finish_node();
        }

        if self.current() == Some(SyntaxKind::OpenBracket) {
            self.builder
                .start_node(SyntaxKind::LiteralAnnotations.into());
            self.bump();

            if self.current() != Some(SyntaxKind::CloseBracket) {
                self.parse_term();
                while let Some(SyntaxKind::Comma) = self.current() {
                    self.bump();
                    self.parse_term();
                }

                match self.current() {
                    Some(SyntaxKind::CloseBracket) => self.bump(),
                    Some(token) => {
                        self.recover(
                            format!("expected ']' to close literal annotation, got {:?}", token),
                            |t| t == SyntaxKind::CloseBracket,
                            |t| t == SyntaxKind::Dot || t == SyntaxKind::Semi,
                        );
                    }
                    None => self.unexpected_eof = true,
                }
            }

            self.builder.finish_node();
        }

        self.builder.finish_node();
    }

    fn parse_term(&mut self) {
        let checkpoint = self.builder.checkpoint();
        self.parse_conjunction();
        while self.current() == Some(SyntaxKind::Or) {
            self.builder
                .start_node_at(checkpoint, SyntaxKind::Disjunction.into());
            self.bump();
            self.parse_conjunction();
            self.builder.finish_node();
        }
    }

    fn parse_conjunction(&mut self) {
        let checkpoint = self.builder.checkpoint();
        self.parse_negation();
        while self.current() == Some(SyntaxKind::And) {
            self.builder
                .start_node_at(checkpoint, SyntaxKind::Conjunction.into());
            self.bump();
            self.parse_negation();
            self.builder.finish_node();
        }
    }

    fn parse_negation(&mut self) {
        if self.current() == Some(SyntaxKind::Not) {
            self.builder.start_node(SyntaxKind::Negation.into());
            self.parse_negation();
            self.builder.finish_node();
        } else {
            self.parse_comparison();
        }
    }

    fn parse_comparison(&mut self) {
        let checkpoint = self.builder.checkpoint();
        self.parse_additive_expression();
        if self
            .current()
            .and_then(|t| t.comparison_operator())
            .is_some()
        {
            self.builder
                .start_node_at(checkpoint, SyntaxKind::Comparison.into());
            self.bump();
            self.parse_additive_expression();
            self.builder.finish_node();
        }
    }

    fn parse_additive_expression(&mut self) {
        let checkpoint = self.builder.checkpoint();
        self.parse_multiplicative_expression();
        while self
            .current()
            .and_then(|t| t.additive_operator())
            .is_some()
        {
            self.builder
                .start_node_at(checkpoint, SyntaxKind::AdditiveExpression.into());
            self.bump();
            self.parse_multiplicative_expression();
            self.builder.finish_node();
        }
    }

    fn parse_multiplicative_expression(&mut self) {
        let checkpoint = self.builder.checkpoint();
        self.parse_unary_expression();
        while self.current().and_then(|t| t.multiplicative_operator()).is_some() {
            self.builder
                .start_node_at(checkpoint, SyntaxKind::MultiplicativeExpression.into());
            self.bump();
            self.parse_unary_expression();
            self.builder.finish_node();
        }
    }

    fn parse_unary_expression(&mut self) {
        if self.current().and_then(|t| t.unary_operator()).is_some() {
            self.builder.start_node(SyntaxKind::UnaryExpression.into());
            self.bump();
            self.parse_unary_expression();
            self.builder.finish_node();
        } else {
            self.parse_exponentiation();
        }
    }

    fn parse_exponentiation(&mut self) {
        let checkpoint = self.builder.checkpoint();
        self.parse_atom();
        while self.current() == Some(SyntaxKind::Pow) {
            self.builder.start_node_at(checkpoint, SyntaxKind::Exponentiation.into());
            self.bump();
            self.parse_unary_expression();
            self.builder.finish_node();
        }
    }

    fn parse_atom(&mut self) {
        match self.current() {
            Some(
                SyntaxKind::Variable
                | SyntaxKind::Wildcard
                | SyntaxKind::Integer
                | SyntaxKind::Float
                | SyntaxKind::True
                | SyntaxKind::False
                | SyntaxKind::String,
            ) => self.bump(),
            Some(SyntaxKind::Functor) => self.parse_literal(),
            Some(SyntaxKind::OpenBracket) => {
                todo!("lists not yet implemented")
            }
            Some(SyntaxKind::OpenParen) => {
                self.bump();
                self.parse_term();
                match self.current() {
                    Some(SyntaxKind::CloseParen) => self.bump(),
                    Some(token) => self.recover(format!("expected ')', got {:?}", token), |t| t == SyntaxKind::CloseParen, |t| t == SyntaxKind::Semi || t == SyntaxKind::Dot),
                    None => self.unexpected_eof = true,
                }
            }
            Some(token) => {
                self.recover(
                    format!("expected atom, got {:?}", token),
                    |_| false,
                    |t| {
                        t == SyntaxKind::Semi || t == SyntaxKind::Dot || t == SyntaxKind::CloseParen
                    },
                );
            }
            None => self.unexpected_eof = true,
        }
    }

    fn recover(
        &mut self,
        message: impl Into<String>,
        mut until_inclusive: impl FnMut(SyntaxKind) -> bool,
        mut until_exclusive: impl FnMut(SyntaxKind) -> bool,
    ) {
        self.push_error(message);
        self.builder.start_node(SyntaxKind::Error.into());
        while let Some(token) = self.current() {
            if until_exclusive(token) {
                break;
            }
            self.bump();
            if until_inclusive(token) {
                break;
            }
        }
        self.builder.finish_node();
    }

    fn push_error(&mut self, message: impl Into<String>) {
        self.errors.push(ParserError {
            message: message.into(),
            token_idx: self.tokens.current_token_idx(),
        });
    }
}
