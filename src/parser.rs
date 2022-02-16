use std::fmt;
use std::iter::Peekable;
use rowan::{GreenNode, GreenNodeBuilder};

use crate::syntax::{LexedStr, SyntaxKind, LexedStrIter};

#[derive(Debug)]
pub struct Parsed {
    pub green_node: GreenNode,
    pub errors: Vec<ParserError>,
}

#[derive(Debug)]
pub struct ParserError(String);

impl ParserError {
    fn unexpected_eof() -> ParserError {
        ParserError("unexpected end of file".to_owned())
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

struct Parser<'a> {
    builder: GreenNodeBuilder<'static>,
    tokens: Peekable<LexedStrIter<'a>>,
    errors: Vec<ParserError>,
}

pub fn parse(lexed: LexedStr<'_>) -> Parsed {
    Parser {
        builder: GreenNodeBuilder::new(),
        tokens: lexed.iter().peekable(),
        errors: Vec::new(),
    }
    .parse()
}

impl Parser<'_> {
    fn skip_noise(&mut self) {
        while let Some((SyntaxKind::Whitespace | SyntaxKind::LineComment | SyntaxKind::BlockComment, _)) = self.tokens.peek() {
            self.bump();
        }
    }

    fn bump(&mut self) {
        let (token, text) = self.tokens.next().unwrap();
        self.builder.token(token.into(), text);
    }

    fn current(&mut self) -> Option<SyntaxKind> {
        self.skip_noise();
        self.tokens.peek().map(|(token, _)| *token)
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
        self.builder.start_node_at(checkpoint, SyntaxKind::Belief.into());
        self.builder.finish_node();
    }

    fn parse_literal(&mut self) {
        self.builder.start_node(SyntaxKind::Literal.into());

        assert!(self.current() == Some(SyntaxKind::Functor));
        self.bump();

        if self.current() == Some(SyntaxKind::OpenParen) {
            self.bump();
            self.builder.start_node(SyntaxKind::LiteralTerms.into());
            self.builder.finish_node();
        }

        self.builder.finish_node();
    }

    fn parse_term(&mut self) {
        self.parse_atom();
    }

    fn parse_atom(&mut self) {
        match self.current() {
            Some(SyntaxKind::Variable | SyntaxKind::Integer | SyntaxKind::Float | SyntaxKind::True | SyntaxKind::False | SyntaxKind::String) => self.bump(),
            Some(SyntaxKind::Functor) => self.parse_literal(),
            Some(SyntaxKind::OpenParen) => {
                todo!()
            }
            Some(SyntaxKind::OpenBracket) => {
                todo!()
            }
            Some(token) => {
                self.bump();
                self.errors.push(ParserError(format!("expected atom, got {:?}", token)));
            },
            None => {
                self.errors.push(ParserError::unexpected_eof());
            }
        }
    }
}
