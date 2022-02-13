pub struct Token {
    pub kind: TokenKind,
    pub len: usize,
}

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
