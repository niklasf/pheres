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
    /// `_`
    Wildcard,
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

    pub fn first(&self) -> char {
        self.chars.clone().next().unwrap_or(EOF_CHAR)
    }

    pub fn second(&self) -> char {
        let mut iter = self.chars.clone();
        iter.next();
        iter.next().unwrap_or(EOF_CHAR)
    }

    pub fn bump(&mut self) -> Option<char> {
        self.chars.next()
    }

    pub fn eat_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
        while predicate(self.first()) && !self.is_eof() {
            self.bump();
        }
    }

    pub fn followed_by(&mut self, s: &str) -> bool {
        if self.chars.as_str().starts_with(s) {
            self.chars = self.chars.as_str()[s.len()..].chars();
            true
        } else {
            false
        }
    }

    pub fn advance_token(&mut self) -> Token {
        Token {
            kind: match self.bump().unwrap() {
                ch if ch.is_whitespace() => self.whitespace(),
                '/' => match self.first() {
                    '/' => self.line_comment(),
                    '*' => self.block_comment(),
                    _ => TokenKind::Slash,
                },
                '#' => self.line_comment(),
                '(' => TokenKind::OpenParen,
                ')' => TokenKind::CloseParen,
                '[' => TokenKind::OpenBracket,
                ']' => TokenKind::CloseBracket,
                '{' => TokenKind::OpenBrace,
                '}' => TokenKind::CloseBrace,
                '!' => match self.first() {
                    '!' => {
                        self.bump();
                        TokenKind::BangBang
                    }
                    _ => TokenKind::Bang,
                },
                '?' => TokenKind::Question,
                ':' => match self.first() {
                    '-' => {
                        self.bump();
                        TokenKind::Define
                    }
                    _ => TokenKind::Colon,
                },
                '<' => match self.first() {
                    '-' => {
                        self.bump();
                        TokenKind::Arrow
                    }
                    '=' => {
                        self.bump();
                        TokenKind::LtEq
                    }
                    _ => TokenKind::Lt,
                },
                '>' => match self.first() {
                    '=' => {
                        self.bump();
                        TokenKind::GtEq
                    }
                    _ => TokenKind::Gt,
                },
                '=' => match (self.first(), self.second()) {
                    ('=', _) => {
                        self.bump();
                        TokenKind::Equal
                    }
                    ('.', '.') => {
                        self.bump();
                        self.bump();
                        TokenKind::Decompose
                    }
                    _ => TokenKind::Eq,
                },
                '*' => match self.first() {
                    '*' => {
                        self.bump();
                        TokenKind::Pow
                    }
                    _ => TokenKind::Star,
                },
                '-' => match self.first() {
                    '+' => {
                        self.bump();
                        TokenKind::MinusPlus
                    }
                    _ => TokenKind::Minus,
                },
                '&' => TokenKind::And,
                '|' => match (self.first(), self.second()) {
                    ('&', '|') => {
                        self.bump();
                        self.bump();
                        TokenKind::ForkJoinAnd
                    }
                    ('|', '|') => {
                        self.bump();
                        self.bump();
                        TokenKind::ForkJoinXor
                    }
                    _ => TokenKind::Or,
                },
                '+' => TokenKind::Plus,
                '.' => if self.first().is_ascii_lowercase() {
                    self.bump();
                    self.functor()
                } else {
                    TokenKind::Dot
                },
                ',' => TokenKind::Comma,
                ';' => TokenKind::Semi,
                '@' => TokenKind::At,
                '\\' if self.followed_by("==") => TokenKind::NotEqual,
                't' if self.followed_by("rue") => TokenKind::True,
                'f' if self.followed_by("alse") => TokenKind::False,
                'i' if self.followed_by("f") => TokenKind::If,
                'e' if self.followed_by("lse") => TokenKind::Else,
                'w' if self.followed_by("hile") => TokenKind::While,
                'f' if self.followed_by("or") => TokenKind::For,
                'i' if self.followed_by("nclude") => TokenKind::Include,
                'b' if self.followed_by("egin") => TokenKind::Begin,
                'e' if self.followed_by("nd") => TokenKind::End,
                'n' if self.followed_by("ot") => TokenKind::Not,
                'd' if self.followed_by("iv") => TokenKind::Div,
                'm' if self.followed_by("od") => TokenKind::Mod,
                ch if ch.is_ascii_uppercase() => self.variable(),
                ch if ch.is_ascii_lowercase() => self.functor(),
                '_' => {
                    self.eat_while(|ch| ch == '_');
                    if self.first().is_ascii_uppercase() {
                        self.bump();
                        self.variable()
                    } else {
                        TokenKind::Wildcard
                    }
                },
                _ => TokenKind::Unknown,
            },
            len: self.len_consumed(),
        }
    }

    pub fn line_comment(&mut self) -> TokenKind {
        self.eat_while(|ch| ch != '\n');
        TokenKind::LineComment
    }

    pub fn block_comment(&mut self) -> TokenKind {
        self.bump(); // `*`
        while let Some(ch) = self.bump() {
            if ch == '*' && self.first() == '/' {
                self.bump();
                return TokenKind::BlockComment { terminated: true }
            }
        }
        TokenKind::BlockComment { terminated: false }
    }

    pub fn whitespace(&mut self) -> TokenKind {
        self.eat_while(char::is_whitespace);
        TokenKind::Whitespace
    }

    pub fn variable(&mut self) -> TokenKind {
        self.eat_while(|ch| ch == '_' || ch.is_ascii_alphanumeric());
        TokenKind::Variable
    }

    pub fn functor(&mut self) -> TokenKind {
        loop {
            self.eat_while(|ch| ch == '_' || ch.is_ascii_alphanumeric());
            if self.first() == '.' && self.second().is_ascii_lowercase() {
                self.bump();
                self.bump();
                continue;
            }
            return TokenKind::Functor;
        }
    }
}
