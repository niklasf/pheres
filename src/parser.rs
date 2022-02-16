use std::fmt;

use rowan::{GreenNode, GreenNodeBuilder};

use crate::syntax::{LexedStr, LexedStrIter, SyntaxKind, TokenIdx};

#[derive(Debug)]
pub struct Parsed {
    pub green_node: GreenNode,
    pub errors: Vec<ParserError>,
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
}

pub fn parse(lexed: &LexedStr<'_>) -> Parsed {
    Parser {
        builder: GreenNodeBuilder::new(),
        tokens: lexed.iter(),
        errors: Vec::new(),
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

    fn eat_while(&mut self, mut predicate: impl FnMut(SyntaxKind) -> bool) {
        while let Some(token) = self.current() {
            if predicate(token) {
                self.bump();
            } else {
                break;
            }
        }
    }

    fn parse(mut self) -> Parsed {
        self.builder.start_node(SyntaxKind::Root.into());

        while let Some(token) = self.current() {
            match token {
                SyntaxKind::Functor => self.parse_rule_or_belief(),
                _ => self.bump(),
            }
        }

        self.builder.finish_node(); // root

        Parsed {
            green_node: self.builder.finish(),
            errors: self.errors,
        }
    }

    fn parse_rule_or_belief(&mut self) {
        let checkpoint = self.builder.checkpoint();
        self.parse_literal();

        if self.current() == Some(SyntaxKind::Define) {
            self.builder
                .start_node_at(checkpoint, SyntaxKind::Rule.into());
            todo!();
        } else {
            self.builder
                .start_node_at(checkpoint, SyntaxKind::Belief.into());
        }

        if self.current() == Some(SyntaxKind::Dot) {
            self.bump();
        } else {
            self.recover("expected '.' after rule or belief", |t| t == SyntaxKind::Dot, |_| false);
        }

        self.builder.finish_node();
    }

    fn parse_literal(&mut self) {
        self.builder.start_node(SyntaxKind::Literal.into());

        assert!(self.current() == Some(SyntaxKind::Functor));
        self.bump();

        if self.current() == Some(SyntaxKind::OpenParen) {
            self.bump();
            self.builder.start_node(SyntaxKind::LiteralTerms.into());

            self.parse_term();
            while let Some(SyntaxKind::Comma) = self.current() {
                self.bump();
                self.parse_term();
            }

            match self.current() {
                Some(SyntaxKind::CloseParen) => self.bump(),
                Some(token) => {
                    self.recover(format!("expected ')' to close literal, got {:?}", token), |t| t == SyntaxKind::CloseParen, |t| t == SyntaxKind::Dot || t == SyntaxKind::Semi);
                    self.bump();
                }
                None => self.push_error("expected ')', got end of file".to_owned()),
            }

            self.builder.finish_node();
        }

        if self.current() == Some(SyntaxKind::OpenBracket) {
            self.bump();
            self.builder.start_node(SyntaxKind::LiteralAnnotations.into());

            if self.current() != Some(SyntaxKind::CloseBracket) {
                self.parse_term();
                while let Some(SyntaxKind::Comma) = self.current() {
                    self.bump();
                    self.parse_term();
                }

                match self.current() {
                    Some(SyntaxKind::CloseBracket) => self.bump(),
                    Some(token) => {
                        self.recover(format!("expected ']' to close literal annotation, got {:?}", token), |t| t == SyntaxKind::CloseBracket, |t| t == SyntaxKind::Dot || t == SyntaxKind::Semi);
                    }
                    None => self.push_error("expected ']', got end of file".to_owned()),
                }
            }

            self.builder.finish_node();
        }

        self.builder.finish_node();
    }

    fn parse_term(&mut self) {
        self.parse_atom();
    }

    fn parse_atom(&mut self) {
        match self.current() {
            Some(
                SyntaxKind::Variable
                | SyntaxKind::Integer
                | SyntaxKind::Float
                | SyntaxKind::True
                | SyntaxKind::False
                | SyntaxKind::String,
            ) => self.bump(),
            Some(SyntaxKind::Functor) => self.parse_literal(),
            Some(SyntaxKind::OpenParen) => {
                todo!()
            }
            Some(SyntaxKind::OpenBracket) => {
                todo!()
            }
            Some(token) => {
                self.bump();
                self.push_error(format!("expected atom, got {:?}", token));
            }
            None => {
                self.push_error("expected atom, got end of file".to_owned());
            }
        }
    }

    fn recover(&mut self, message: impl Into<String>, mut until_inclusive: impl FnMut(SyntaxKind) -> bool, mut until_exclusive: impl FnMut(SyntaxKind) -> bool) {
        self.push_error(message.into());
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

    fn push_error(&mut self, message: String) {
        self.errors.push(ParserError {
            message,
            token_idx: self.tokens.current_token_idx(),
        });
    }
}
