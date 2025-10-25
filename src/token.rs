use logos::Logos;

/// Defines the set of recognizable tokens in the oxython language.
/// The `#[derive(Logos)]` macro from the `logos` crate generates the lexer implementation.
#[derive(Logos, Debug, Clone, PartialEq, Default)]
#[logos(skip r"[ \t\n\f]+")] // Ignore whitespace
pub enum Token {
    // Literals
    #[regex("[0-9]+\\.[0-9]+", |lex| lex.slice().parse::<f64>().ok())]
    Float(f64),

    #[regex("[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    Integer(i64),

    // Handles both single and double-quoted strings.
    #[regex(r#""(?:[^"\\]|\\.)*"|'[^']*'"#, |lex| {
        let slice = lex.slice();
        // Slice the string to remove the opening and closing quotes.
        slice[1..slice.len() - 1].to_string()
    })]
    String(String),

    // Identifiers and keywords. Logos processes variants in order, so keywords
    // must come before the general Identifier regex.
    #[token("print")]
    Print,

    #[token("for")]
    For,

    #[token("while")]
    While,

    #[token("if")]
    If,

    #[token("else")]
    Else,

    #[token("def")]
    Def,

    #[token("return")]
    Return,

    #[token("break")]
    Break,

    #[token("nonlocal")]
    Nonlocal,

    #[token("True")]
    True,

    #[token("False")]
    False,

    #[token("in")]
    In,

    #[token("==")]
    EqualEqual,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    // Operators
    #[token("=")]
    Assign,

    #[token("+=")]
    PlusEqual,

    #[token("*=")]
    StarEqual,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Star,

    #[token("/")]
    Slash,

    #[token("%")]
    Percent,

    #[token("<")]
    Less,

    // Punctuation
    #[token(";")]
    Semicolon,

    #[token(",")]
    Comma,

    #[token(".")]
    Dot,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("[")]
    LBracket,

    #[token("]")]
    RBracket,

    #[token("{")]
    LBrace,

    #[token("}")]
    RBrace,

    #[token(":")]
    Colon,

    #[default]
    Unknown,
}
