use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFiles,
    term,
    term::termcolor::{ColorChoice, StandardStream},
};

mod lexer;
mod parser;
mod syntax;

use crate::{
    parser::parse,
    syntax::{LexedStr, SyntaxErrorKind},
};

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

    let parsed = parse(lexed);
    println!("{}", parsed.green_node);
}
