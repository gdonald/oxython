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
