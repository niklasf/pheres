use codespan_reporting::files::SimpleFiles;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::term::termcolor::{StandardStream, ColorChoice};
use codespan_reporting::term;

mod lexer;
mod syntax;

use syntax::LexedStr;

fn main() {
    let mut files = SimpleFiles::new();

    let source = include_str!("../test.asl");
    let file_id = files.add("test.asl", source);

    let lexed = LexedStr::new(source);

    let diagnostic_stream = StandardStream::stderr(ColorChoice::Auto);
    let diagnostic_config = term::Config::default();

    for error in &lexed.errors {
        let diagnostic = Diagnostic::error()
            .with_message("syntax error")
            .with_labels(vec![
                Label::primary(file_id, lexed.token_range(error.token_idx)).with_message("unterminated or unknown")
            ]);

        term::emit(&mut diagnostic_stream.lock(), &diagnostic_config, &files, &diagnostic).unwrap();
    }
}
