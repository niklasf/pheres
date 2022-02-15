use rowan::Language;

use crate::lexer::{tokenize, TokenKind};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum AgentSpeakLanguage {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
enum SyntaxKind {
    Whitespace,
    LineComment,
    BlockComment,

    Functor,
    Variable,
    Wildcard,
    Integer,
    Float,
    String,

    True,
    False,

    If,
    Else,
    While,
    For,

    Include,
    Begin,
    End,

    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    OpenBrace,
    CloseBrace,

    Arrow,
    Define,
    Colon,

    ForkJoinAnd,
    ForkJoinXor,

    BangBang,
    Bang,
    Question,
    MinusPlus,

    Not,
    Plus,
    Minus,
    Slash,
    Div,
    Mod,
    Pow,
    Star,
    And,
    Or,

    LtEq,
    GtEq,
    NotEqual,
    Equal,
    Decompose,
    Eq,
    Lt,
    Gt,

    Semi,
    Comma,
    Dot,
    At,

    Unknown,
    Eof,

    Const,
    Literal,
    List,
    Rule,
    Goal,
    Formula,
    UnaryOp,
    BinaryOp,
    Plan,
    Event,
    Body,
    WhileLoop,
    ForLoop,
    IfThenElse,
    Root,
}

impl Language for AgentSpeakLanguage {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        // SAFETY: Enum is #[repr(u16)] with Root being the last variant.
        assert!(raw.0 <= SyntaxKind::Root as u16);
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(kind as u16)
    }
}

struct ParserError {
    kind: ParserErrorKind,
    token: usize,
}

enum ParserErrorKind {
    UnterminatedBlockComment,
    UnterminatedString,
    UnexpectedToken,
}

struct LexedStr<'a> {
    text: &'a str,
    kind: Vec<SyntaxKind>,
    start: Vec<usize>,
    errors: Vec<ParserError>,
}

impl LexedStr<'_> {
    pub fn new(text: &str) -> LexedStr<'_> {
        let mut res = LexedStr {
            text,
            kind: Vec::new(),
            start: Vec::new(),
            errors: Vec::new(),
        };

        let mut offset = 0;

        for token in tokenize(text) {
            let syntax_kind = match token.kind {
                TokenKind::Whitespace => SyntaxKind::Whitespace,
                TokenKind::LineComment => SyntaxKind::LineComment,
                TokenKind::BlockComment { terminated } => {
                    if !terminated {
                        res.errors.push(ParserError {
                            kind: ParserErrorKind::UnterminatedBlockComment,
                            token: res.kind.len(),
                        });
                    }
                    SyntaxKind::BlockComment
                }

                TokenKind::Functor => SyntaxKind::Functor,
                TokenKind::Variable => SyntaxKind::Variable,
                TokenKind::Wildcard => SyntaxKind::Wildcard,
                TokenKind::Integer => SyntaxKind::Integer,
                TokenKind::Float => SyntaxKind::Float,
                TokenKind::String { terminated } => {
                    if !terminated {
                        res.errors.push(ParserError {
                            kind: ParserErrorKind::UnterminatedString,
                            token: res.kind.len(),
                        });
                    }
                    SyntaxKind::String
                }

                TokenKind::True => SyntaxKind::True,
                TokenKind::False => SyntaxKind::False,

                TokenKind::If => SyntaxKind::If,
                TokenKind::Else => SyntaxKind::Else,
                TokenKind::While => SyntaxKind::While,
                TokenKind::For => SyntaxKind::For,

                TokenKind::Include => SyntaxKind::Include,
                TokenKind::Begin => SyntaxKind::Begin,
                TokenKind::End => SyntaxKind::End,

                TokenKind::OpenParen => SyntaxKind::OpenParen,
                TokenKind::CloseParen => SyntaxKind::CloseParen,
                TokenKind::OpenBracket => SyntaxKind::OpenBracket,
                TokenKind::CloseBracket => SyntaxKind::CloseBracket,
                TokenKind::OpenBrace => SyntaxKind::OpenBrace,
                TokenKind::CloseBrace => SyntaxKind::CloseBrace,

                TokenKind::Arrow => SyntaxKind::Arrow,
                TokenKind::Define => SyntaxKind::Define,
                TokenKind::Colon => SyntaxKind::Colon,

                TokenKind::ForkJoinAnd => SyntaxKind::ForkJoinAnd,
                TokenKind::ForkJoinXor => SyntaxKind::ForkJoinXor,

                TokenKind::BangBang => SyntaxKind::BangBang,
                TokenKind::Bang => SyntaxKind::Bang,
                TokenKind::Question => SyntaxKind::Question,
                TokenKind::MinusPlus => SyntaxKind::MinusPlus,

                TokenKind::Not => SyntaxKind::Not,
                TokenKind::Plus => SyntaxKind::Plus,
                TokenKind::Minus => SyntaxKind::Minus,
                TokenKind::Slash => SyntaxKind::Slash,
                TokenKind::Div => SyntaxKind::Div,
                TokenKind::Mod => SyntaxKind::Mod,
                TokenKind::Pow => SyntaxKind::Pow,
                TokenKind::Star => SyntaxKind::Star,
                TokenKind::And => SyntaxKind::And,
                TokenKind::Or => SyntaxKind::Or,

                TokenKind::LtEq => SyntaxKind::LtEq,
                TokenKind::GtEq => SyntaxKind::GtEq,
                TokenKind::NotEqual => SyntaxKind::NotEqual,
                TokenKind::Equal => SyntaxKind::Equal,
                TokenKind::Decompose => SyntaxKind::Decompose,
                TokenKind::Eq => SyntaxKind::Eq,
                TokenKind::Lt => SyntaxKind::Lt,
                TokenKind::Gt => SyntaxKind::Gt,

                TokenKind::Semi => SyntaxKind::Semi,
                TokenKind::Comma => SyntaxKind::Comma,
                TokenKind::Dot => SyntaxKind::Dot,
                TokenKind::At => SyntaxKind::At,

                TokenKind::Unknown => {
                    res.errors.push(ParserError {
                        kind: ParserErrorKind::UnexpectedToken,
                        token: res.kind.len(),
                    });
                    SyntaxKind::Unknown
                }
            };

            res.kind.push(syntax_kind);
            res.start.push(offset);
            offset += token.len;
        }

        res.kind.push(SyntaxKind::Eof);

        res
    }
}
