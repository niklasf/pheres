mod lexer;

fn main() {
    for token in lexer::tokenize("abc // hello") {
        dbg!(token);
    }
}
