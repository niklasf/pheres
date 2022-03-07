use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFiles,
    term,
    term::termcolor::{ColorChoice, StandardStream},
};
use rowan::NodeOrToken;

mod lexer;
mod parser;
mod syntax;
mod runtime;

use crate::{
    parser::parse,
    syntax::{LexedStr, SyntaxElement, SyntaxKind, SyntaxNode},
};

fn print(level: usize, element: SyntaxElement) {
    let kind: SyntaxKind = element.kind().into();
    print!("{:indent$}", "", indent = level * 2);
    match element {
        NodeOrToken::Node(node) => {
            println!("- {:?}", kind);
            for child in node.children_with_tokens() {
                print(level + 1, child);
            }
        }
        NodeOrToken::Token(token) => println!("- {:?} {:?}", token.text(), kind),
    }
}

fn main() {
    let mut files = SimpleFiles::new();

    let source = include_str!("../test.asl");
    let file_id = files.add("test.asl", source);

    let lexed = LexedStr::new(source);

    let diagnostic_stream = StandardStream::stderr(ColorChoice::Auto);
    let diagnostic_config = term::Config::default();

    for error in &lexed.errors {
        let diagnostic = Diagnostic::error()
            .with_message(error.kind.to_string())
            .with_labels(vec![Label::primary(
                file_id,
                lexed.token_range(error.token_idx),
            )]);
        term::emit(
            &mut diagnostic_stream.lock(),
            &diagnostic_config,
            &files,
            &diagnostic,
        )
        .unwrap();
    }

    let parsed = parse(&lexed);

    for error in &parsed.errors {
        let diagnostic = Diagnostic::error()
            .with_message(error.to_string())
            .with_labels(vec![Label::primary(
                file_id,
                lexed.token_range(error.token_idx),
            )]);
        term::emit(
            &mut diagnostic_stream.lock(),
            &diagnostic_config,
            &files,
            &diagnostic,
        )
        .unwrap();
    }

    if parsed.unexpected_eof {
        let last = lexed.text.len() - 1;
        let diagnostic = Diagnostic::error()
            .with_message("unexpected end of file")
            .with_labels(vec![Label::primary(file_id, last..last)]);
        term::emit(
            &mut diagnostic_stream.lock(),
            &diagnostic_config,
            &files,
            &diagnostic,
        )
        .unwrap();
    }

    print(0, SyntaxNode::new_root(parsed.green_node).into());
}
