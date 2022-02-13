use std::iter;
use std::mem;
use std::str::Chars;

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub len: usize,
}

#[derive(Debug)]
pub enum TokenKind {
    /// One or more whitespace characters
    Whitespace,
    /// `// comment` or `# comment`
    LineComment,
    /// `/* comment */`
    BlockComment { terminated: bool },

    /// `foo`
    Functor,
    /// `Foo`
    Variable,
    /// `42e-3`
    Number,
    /// `"foo\n"`
    String,

    /// `true`
    True,
    /// `false`
    False,

    /// `if`
    If,
    /// `else`
    Else,
    /// `while`
    While,
    /// `for`
    For,

    /// `include`
    Include,
    /// `begin`
    Begin,
    /// `end`
    End,

    /// `(`
    OpenParen,
    /// `)`
    CloseParen,
    /// `[`
    OpenBracket,
    /// `]`
    CloseBracket,
    /// `{`
    OpenBrace,
    /// `}`
    CloseBrace,

    /// `<-`
    Arrow,
    /// `:-`
    Define,
    /// `:`,
    Colon,

    /// `|&|`
    ForkJoinAnd,
    /// `|||`
    ForkJoinXor,

    /// `!!`
    BangBang,
    /// `!`,
    Bang,
    /// `?`
    Question,
    /// `-+`
    MinusPlus,

    /// `not`
    Not,
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `/`
    Slash,
    /// `div`
    Div,
    /// `mod`
    Mod,
    /// `**`
    Pow,
    /// `*`
    Star,
    /// `&`
    And,
    /// `|`
    Or,

    /// `<=`
    LtEq,
    /// `>=`
    GtEq,
    /// `\==`
    NotEqual,
    /// `==`
    Equal,
    /// `=..`
    Decompose,
    /// `=`
    Eq,
    /// `<`
    Lt,
    /// `>`
    Gt,

    /// `;`
    Semi,
    /// `,`
    Comma,
    /// `.`
    Dot,
    /// `@`
    At,

    /// Unkown token
    Unknown,
}

pub fn tokenize(input: &str) -> impl Iterator<Item = Token> + '_ {
    let mut cursor = Cursor::new(input);
    iter::from_fn(move || {
        if cursor.is_eof() {
            None
        } else {
            cursor.reset_len_consumed();
            Some(cursor.advance_token())
        }
    })
}

const EOF_CHAR: char = '\0';

struct Cursor<'a> {
    initial_len: usize,
    chars: Chars<'a>,
}

impl Cursor<'_> {
    pub fn new(input: &str) -> Cursor<'_> {
        Cursor {
            initial_len: input.len(),
            chars: input.chars(),
        }
    }

    pub fn is_eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }

    pub fn reset_len_consumed(&mut self) {
        self.initial_len = self.chars.as_str().len();
    }

    pub fn len_consumed(&self) -> usize {
        self.initial_len - self.chars.as_str().len()
    }

    pub fn peek(&self) -> char {
        self.chars.clone().next().unwrap_or(EOF_CHAR)
    }

    pub fn bump(&mut self) -> char {
        self.chars.next().unwrap()
    }

    pub fn eat_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
        while predicate(self.peek()) && !self.is_eof() {
            self.bump();
        }
    }

    pub fn advance_token(&mut self) -> Token {
        Token {
            kind: match self.bump() {
                '/' => match self.peek() {
                    '/' => self.line_comment(),
                    '*' => self.block_comment(),
                    _ => TokenKind::Slash,
                },
                _ => TokenKind::Unknown,
            },
            len: self.len_consumed(),
        }
    }

    pub fn line_comment(&mut self) -> TokenKind {
        self.eat_while(|c| c != '\n');
        TokenKind::LineComment
    }

    pub fn block_comment(&mut self) -> TokenKind {
        todo!()
    }
}
