use rowan::{GreenNode, GreenNodeBuilder};

use crate::syntax::{LexedStr, SyntaxKind, LexedStrIter};

#[derive(Debug)]
pub struct Parsed {
    green_node: GreenNode,
    errors: Vec<ParserError>,
}

#[derive(Debug)]
pub struct ParserError {}

struct Parser<'a> {
    builder: GreenNodeBuilder<'static>,
    tokens: LexedStrIter<'a>,
    errors: Vec<ParserError>,
}

pub fn parse(lexed: LexedStr<'_>) -> Parsed {
    Parser {
        builder: GreenNodeBuilder::new(),
        tokens: lexed.iter(),
        errors: Vec::new(),
    }
    .parse()
}

impl Parser<'_> {
    fn parse(mut self) -> Parsed {
        self.builder.start_node(SyntaxKind::Root.into());

        self.builder.finish_node(); // root

        Parsed {
            green_node: self.builder.finish(),
            errors: self.errors,
        }
    }
}
