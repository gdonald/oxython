use logos::Logos;
use oxython::token::Token;

#[test]
fn test_lexer() {
    let input = r#"
        a = 1
        b = 2
        c = a + b
        print('c: ', c)
        print("hello world")
    "#;

    let lexer = Token::lexer(input);
    let tokens: Vec<Token> = lexer.filter_map(Result::ok).collect();

    let expected_tokens = vec![
        Token::Identifier("a".to_string()),
        Token::Assign,
        Token::Integer(1),
        Token::Identifier("b".to_string()),
        Token::Assign,
        Token::Integer(2),
        Token::Identifier("c".to_string()),
        Token::Assign,
        Token::Identifier("a".to_string()),
        Token::Plus,
        Token::Identifier("b".to_string()),
        Token::Print,
        Token::LParen,
        Token::String("c: ".to_string()),
        Token::Comma,
        Token::Identifier("c".to_string()),
        Token::RParen,
        Token::Print,
        Token::LParen,
        Token::String("hello world".to_string()),
        Token::RParen,
    ];

    assert_eq!(tokens, expected_tokens);
}

#[test]
fn test_type_annotations() {
    let input = r#"
        x: int = 5
        y: float = 3.14
        name: str = "test"
        flag: bool = True
        items: list
        data: dict
        coords: tuple
    "#;

    let lexer = Token::lexer(input);
    let tokens: Vec<Token> = lexer.filter_map(Result::ok).collect();

    let expected_tokens = vec![
        // x: int = 5
        Token::Identifier("x".to_string()),
        Token::Colon,
        Token::Identifier("int".to_string()),
        Token::Assign,
        Token::Integer(5),
        // y: float = 3.14
        Token::Identifier("y".to_string()),
        Token::Colon,
        Token::Identifier("float".to_string()),
        Token::Assign,
        Token::Float(3.14),
        // name: str = "test"
        Token::Identifier("name".to_string()),
        Token::Colon,
        Token::Identifier("str".to_string()),
        Token::Assign,
        Token::String("test".to_string()),
        // flag: bool = True
        Token::Identifier("flag".to_string()),
        Token::Colon,
        Token::Identifier("bool".to_string()),
        Token::Assign,
        Token::True,
        // items: list
        Token::Identifier("items".to_string()),
        Token::Colon,
        Token::Identifier("list".to_string()),
        // data: dict
        Token::Identifier("data".to_string()),
        Token::Colon,
        Token::Identifier("dict".to_string()),
        // coords: tuple
        Token::Identifier("coords".to_string()),
        Token::Colon,
        Token::Identifier("tuple".to_string()),
    ];

    assert_eq!(tokens, expected_tokens);
}

#[test]
fn test_function_type_annotations() {
    let input = "def add(a: int, b: int) -> int:";

    let lexer = Token::lexer(input);
    let tokens: Vec<Token> = lexer.filter_map(Result::ok).collect();

    let expected_tokens = vec![
        Token::Def,
        Token::Identifier("add".to_string()),
        Token::LParen,
        Token::Identifier("a".to_string()),
        Token::Colon,
        Token::Identifier("int".to_string()),
        Token::Comma,
        Token::Identifier("b".to_string()),
        Token::Colon,
        Token::Identifier("int".to_string()),
        Token::RParen,
        Token::Arrow,
        Token::Identifier("int".to_string()),
        Token::Colon,
    ];

    assert_eq!(tokens, expected_tokens);
}
