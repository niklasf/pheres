use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFiles,
    term,
    term::termcolor::{ColorChoice, StandardStream},
};

mod lexer;
mod syntax;

use crate::syntax::{LexedStr, SyntaxErrorKind};

fn main() {
    let mut files = SimpleFiles::new();

    let source = include_str!("../test.asl");
    let file_id = files.add("test.asl", source);

    let lexed = LexedStr::new(source);

    let diagnostic_stream = StandardStream::stderr(ColorChoice::Auto);
    let diagnostic_config = term::Config::default();

    for error in &lexed.errors {
        let diagnostic = Diagnostic::error()
            .with_message(match error.kind {
                SyntaxErrorKind::UnexpectedToken => "unexpected token",
                SyntaxErrorKind::UnterminatedString => "unterminated string",
                SyntaxErrorKind::UnterminatedBlockComment => "unterminated block comment",
            })
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
}
