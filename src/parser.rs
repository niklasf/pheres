use rowan::Language;

use crate::lexer::TokenKind;

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
