use std::{fmt, ops::Range};

use rowan::Language;

use crate::lexer::{tokenize, TokenKind};

#[derive(Copy, Clone, Debug)]
pub struct TokenIdx(usize);

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AgentSpeakLanguage {}

pub type SyntaxNode = rowan::SyntaxNode<AgentSpeakLanguage>;
pub type SyntaxToken = rowan::SyntaxToken<AgentSpeakLanguage>;
pub type SyntaxElement = rowan::NodeOrToken<SyntaxNode, SyntaxToken>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum SyntaxKind {
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
    Tilde,
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

    Error,
    Eof,

    Belief,
    Rule,
    InitialGoal,
    Plan,
    PlanAnnotation,
    PlanContext,
    Body,
    Formula,
    Literal,
    LiteralTerms,
    LiteralAnnotations,
    Disjunction,
    Conjunction,
    Negation,
    Comparison,
    AdditiveExpression,
    MultiplicativeExpression,
    UnaryExpression,
    PowerExpression,
    Exponentiation,
    Atom,
    List,
    WhileLoop,
    ForLoop,
    IfThenElse,
    Root, // last variant
}

impl SyntaxKind {
    pub fn comparison_operator(self) -> Option<ComparisonOperator> {
        Some(match self {
            SyntaxKind::LtEq => ComparisonOperator::LtEq,
            SyntaxKind::GtEq => ComparisonOperator::GtEq,
            SyntaxKind::NotEqual => ComparisonOperator::NotEqual,
            SyntaxKind::Equal => ComparisonOperator::Equal,
            SyntaxKind::Decompose => ComparisonOperator::Decompose,
            SyntaxKind::Eq => ComparisonOperator::Eq,
            SyntaxKind::Lt => ComparisonOperator::Lt,
            SyntaxKind::Gt => ComparisonOperator::Gt,
            _ => return None,
        })
    }

    pub fn additive_operator(self) -> Option<AdditiveOperator> {
        Some(match self {
            SyntaxKind::Plus => AdditiveOperator::Add,
            SyntaxKind::Minus => AdditiveOperator::Sub,
            _ => return None,
        })
    }

    pub fn multiplicative_operator(self) -> Option<MultiplicativeOperator> {
        Some(match self {
            SyntaxKind::Star => MultiplicativeOperator::Mul,
            SyntaxKind::Slash => MultiplicativeOperator::Div,
            SyntaxKind::Div => MultiplicativeOperator::FloorDiv,
            SyntaxKind::Mod => MultiplicativeOperator::Mod,
            _ => return None,
        })
    }

    pub fn unary_operator(self) -> Option<UnaryOperator> {
        Some(match self {
            SyntaxKind::Plus => UnaryOperator::Pos,
            SyntaxKind::Minus => UnaryOperator::Neg,
            _ => return None,
        })
    }

    pub fn formula_type(self) -> Option<FormulaType> {
        Some(match self {
            SyntaxKind::BangBang => FormulaType::AchieveLater,
            SyntaxKind::Bang => FormulaType::Achieve,
            SyntaxKind::Question => FormulaType::Test,
            SyntaxKind::MinusPlus => FormulaType::Replace,
            SyntaxKind::Plus => FormulaType::Add,
            SyntaxKind::Minus => FormulaType::Remove,
            _ => return None,
        })
    }
}

pub enum ComparisonOperator {
    LtEq,
    GtEq,
    NotEqual,
    Equal,
    Decompose,
    Eq,
    Lt,
    Gt,
}

pub enum FormulaType {
    AchieveLater,
    Achieve,
    Test,
    Replace,
    Remove,
    Add,
    Term,
}

pub enum AdditiveOperator {
    Add,
    Sub,
}

pub enum MultiplicativeOperator {
    Mul,
    Div,
    FloorDiv,
    Mod,
}

pub enum UnaryOperator {
    Pos,
    Neg,
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

impl Language for AgentSpeakLanguage {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        // SAFETY: Enum is #[repr(u16)] with Root being the last variant.
        assert!(raw.0 <= SyntaxKind::Root as u16);
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

#[derive(Debug)]
pub struct SyntaxError {
    pub kind: SyntaxErrorKind,
    pub token_idx: TokenIdx,
}

#[derive(Debug)]
pub enum SyntaxErrorKind {
    UnterminatedBlockComment,
    UnterminatedString,
    UnexpectedToken,
}

impl fmt::Display for SyntaxErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match *self {
            SyntaxErrorKind::UnexpectedToken => "unexpected token",
            SyntaxErrorKind::UnterminatedString => "unterminated string",
            SyntaxErrorKind::UnterminatedBlockComment => "unterminated block comment",
        })
    }
}

#[derive(Debug)]
pub struct LexedStr<'a> {
    pub text: &'a str,
    kind: Vec<SyntaxKind>,
    start: Vec<usize>,
    pub errors: Vec<SyntaxError>,
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
                        res.errors.push(SyntaxError {
                            kind: SyntaxErrorKind::UnterminatedBlockComment,
                            token_idx: TokenIdx(res.kind.len()),
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
                        res.errors.push(SyntaxError {
                            kind: SyntaxErrorKind::UnterminatedString,
                            token_idx: TokenIdx(res.kind.len()),
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
                TokenKind::Tilde => SyntaxKind::Tilde,
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
                    res.errors.push(SyntaxError {
                        kind: SyntaxErrorKind::UnexpectedToken,
                        token_idx: TokenIdx(res.kind.len()),
                    });
                    SyntaxKind::Error
                }
            };

            res.kind.push(syntax_kind);
            res.start.push(offset);
            offset += token.len;
        }

        res.kind.push(SyntaxKind::Eof);
        res.start.push(offset);
        res.start.push(offset);

        res
    }

    pub fn len(&self) -> usize {
        self.kind.len() - 1
    }

    pub fn token_range(&self, idx: TokenIdx) -> Range<usize> {
        self.start[idx.0]..self.start[idx.0 + 1]
    }

    pub fn iter(&self) -> LexedStrIter<'_> {
        LexedStrIter {
            lexed: self,
            token_idx: TokenIdx(0),
        }
    }
}

#[derive(Clone)]
pub struct LexedStrIter<'a> {
    lexed: &'a LexedStr<'a>,
    token_idx: TokenIdx,
}

impl<'a> Iterator for LexedStrIter<'a> {
    type Item = (SyntaxKind, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.peek();
        if res.is_some() {
            self.token_idx = TokenIdx(self.token_idx.0 + 1);
        }
        res
    }
}

impl<'a> LexedStrIter<'a> {
    pub fn current_token_idx(&self) -> TokenIdx {
        self.token_idx
    }

    pub fn peek(&self) -> Option<(SyntaxKind, &'a str)> {
        (self.token_idx.0 < self.lexed.len()).then(|| {
            (
                self.lexed.kind[self.token_idx.0],
                &self.lexed.text[self.lexed.token_range(self.token_idx)],
            )
        })
    }
}
