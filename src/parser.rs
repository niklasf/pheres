use std::iter::Peekable;
use rowan::{GreenNode, GreenNodeBuilder};

use crate::syntax::{LexedStr, SyntaxKind, LexedStrIter};

#[derive(Debug)]
pub struct Parsed {
    pub green_node: GreenNode,
    pub errors: Vec<ParserError>,
}

#[derive(Debug)]
pub enum ParserError {
    ExpectedFunctor { },
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
        //self.builder.start_node(SyntaxKind::RuleOrBelief);
        //self.bump();
        //self.builder.finish_node();
        self.parse_literal();
    }

    fn parse_literal(&mut self) {
        self.builder.start_node(SyntaxKind::Literal.into());

        self.builder.finish_node();
    }
}
