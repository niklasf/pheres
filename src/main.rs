mod lexer;
mod parser;

fn main() {
    for token in lexer::tokenize(include_str!("../test.asl")) {
        dbg!(token);
    }
}
