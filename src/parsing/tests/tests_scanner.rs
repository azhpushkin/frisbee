use crate::parsing::scanner::{scan_tokens, ScannedToken, Token};

fn scan_tokens_helper(s: &str) -> Vec<Token> {
    let (tokens, scan_status) = scan_tokens(s);
    assert!(scan_status.is_ok(), "Error on scanning: {:?}", scan_status);

    let mut tokens_without_pos = tokens.iter().map(|t| t.token.clone()).collect::<Vec<Token>>();
    assert_eq!(tokens_without_pos.last().unwrap(), &Token::EOF);

    tokens_without_pos.truncate(tokens_without_pos.len() - 1);
    tokens_without_pos
}

#[test]
fn test_positions() {
    let test_string = r#" 123 . [] "hey" 888.888 "#;
    let (res, status) = scan_tokens(test_string);

    assert!(status.is_ok());

    assert_eq!(
        res,
        vec![
            ScannedToken { token: Token::Integer(123), first: 1, last: 3 },
            ScannedToken { token: Token::Dot, first: 5, last: 5 },
            ScannedToken { token: Token::LeftSquareBrackets, first: 7, last: 7 },
            ScannedToken { token: Token::RightSquareBrackets, first: 8, last: 8 },
            ScannedToken { token: Token::String(String::from("hey")), first: 10, last: 14 },
            ScannedToken { token: Token::Float(888.888), first: 16, last: 22 },
            ScannedToken { token: Token::EOF, first: 23, last: 23 },
        ]
    );
}

#[test]
fn test_brackets() {
    assert_eq!(
        scan_tokens_helper(r#"()[]}{"#),
        vec![
            Token::LeftParenthesis,
            Token::RightParenthesis,
            Token::LeftSquareBrackets,
            Token::RightSquareBrackets,
            Token::RightCurlyBrackets,
            Token::LeftCurlyBrackets,
        ]
    );
}

#[test]
fn test_operators() {
    assert_eq!(
        scan_tokens_helper("+ - / * . ? ; ,"),
        vec![
            Token::Plus,
            Token::Minus,
            Token::Slash,
            Token::Star,
            Token::Dot,
            Token::Question,
            Token::Semicolon,
            Token::Comma,
        ]
    );
    assert_eq!(
        scan_tokens_helper("<= < > >= = == !=!"),
        vec![
            Token::LessEqual,
            Token::Less,
            Token::Greater,
            Token::GreaterEqual,
            Token::Equal,
            Token::EqualEqual,
            Token::BangEqual,
            Token::Bang,
        ]
    );
}

#[test]
fn test_comments() {
    assert_eq!(scan_tokens_helper("///"), vec![]);
    assert_eq!(scan_tokens_helper("/ //"), vec![Token::Slash]);
    assert_eq!(
        scan_tokens_helper("/ / /"),
        vec![Token::Slash, Token::Slash, Token::Slash]
    );
    assert_eq!(scan_tokens_helper("//\n / //"), vec![Token::Slash]);

    assert_eq!(
        scan_tokens_helper("/* *  / */  * /"),
        vec![Token::Star, Token::Slash]
    );
    // assert_eq!(scan_tokens_helper("/* */"), vec![Token::Star, Token::Slash]);
}

#[test]
fn test_empty_string() {
    assert_eq!(
        scan_tokens_helper(r#""""#),
        vec![Token::String(String::from(""))]
    );
    assert_eq!(
        scan_tokens_helper(r#"   ""   "#),
        vec![Token::String(String::from(""))]
    );
    assert_eq!(
        scan_tokens_helper(r#"  + ""  + "#),
        vec![Token::Plus, Token::String(String::from("")), Token::Plus]
    );
}

#[test]
fn test_simple_string() {
    assert_eq!(
        scan_tokens_helper(r#"  "Hello world!"  "#),
        vec![Token::String(String::from("Hello world!")),]
    );
    assert_eq!(
        scan_tokens_helper(r#".  "+-//*"  ."#),
        vec![Token::Dot, Token::String(String::from("+-//*")), Token::Dot,]
    );
}

// TODO: check panic and errors

#[test]
fn test_numbers() {
    assert_eq!(scan_tokens_helper("123.098"), vec![Token::Float(123.098)],);
    assert_eq!(
        scan_tokens_helper("-0.098"),
        vec![Token::Minus, Token::Float(0.098)],
    );

    assert_eq!(scan_tokens_helper("123"), vec![Token::Integer(123)],);
    assert_eq!(scan_tokens_helper("0"), vec![Token::Integer(0)],);
    assert_eq!(
        scan_tokens_helper("-0"),
        vec![Token::Minus, Token::Integer(0)],
    );

    assert_eq!(
        scan_tokens_helper("0.0 + 999 - -23.0"),
        vec![
            Token::Float(0.0),
            Token::Plus,
            Token::Integer(999),
            Token::Minus,
            Token::Minus,
            Token::Float(23.0),
        ]
    );
}

#[test]
fn test_identifiers() {
    assert_eq!(
        scan_tokens_helper("asd AsD as3_F_8 If Else Spawn"),
        vec![
            Token::Identifier(String::from("asd")),
            Token::TypeIdentifier(String::from("AsD")),
            Token::Identifier(String::from("as3_F_8")),
            Token::TypeIdentifier(String::from("If")),
            Token::TypeIdentifier(String::from("Else")),
            Token::TypeIdentifier(String::from("Spawn")),
        ]
    );
}

#[test]
fn test_own_identifiers() {
    assert_eq!(
        scan_tokens_helper("@field @method_12()"),
        vec![
            Token::OwnIdentifier("field".into()),
            Token::OwnIdentifier("method_12".into()),
            Token::LeftParenthesis,
            Token::RightParenthesis,
        ]
    );

    assert!(scan_tokens("@").1.is_err());
    assert!(scan_tokens("@ field").1.is_err());
    assert!(scan_tokens("(@)field").1.is_err());
    assert!(scan_tokens("@.field").1.is_err());
    assert!(scan_tokens("@2field").1.is_err());
}

#[test]
fn test_keywords() {
    assert_eq!(
        scan_tokens_helper("if else spawn active class"),
        vec![Token::If, Token::Else, Token::Spawn, Token::Active, Token::Class,]
    );
}

#[test]
fn test_loop_keywords() {
    assert_eq!(
        scan_tokens_helper("while foreach break continue in void"),
        vec![
            Token::While,
            Token::Foreach,
            Token::Break,
            Token::Continue,
            Token::In,
            Token::Void
        ]
    );
}

#[test]
fn test_question_next_token() {
    assert_eq!(
        scan_tokens_helper("Int? "),
        vec![Token::TypeIdentifier(String::from("Int")), Token::Question,]
    );
    assert_eq!(
        scan_tokens_helper("(Int?)"),
        vec![
            Token::LeftParenthesis,
            Token::TypeIdentifier(String::from("Int")),
            Token::Question,
            Token::RightParenthesis,
        ]
    );
    assert_eq!(
        scan_tokens_helper("String?,"),
        vec![Token::TypeIdentifier(String::from("String")), Token::Question, Token::Comma]
    );
    assert_eq!(
        scan_tokens_helper("[Actor?]"),
        vec![
            Token::LeftSquareBrackets,
            Token::TypeIdentifier(String::from("Actor")),
            Token::Question,
            Token::RightSquareBrackets,
        ]
    );
}


#[test]
fn test_question_dot_and_elvis() {
    assert_eq!(
        scan_tokens_helper("? ?:-1 Int?.("),
        vec![
            Token::Question,
            Token::QuestionElvis,
            Token::Minus,
            Token::Integer(1),
            Token::TypeIdentifier(String::from("Int")),
            Token::QuestionDot,
            Token::LeftParenthesis,
        ]
    );
}

#[test]
fn ensure_not_alpha_after_question() {
    let (_, scan_status) = scan_tokens(&String::from("Int?asd"));
    assert!(scan_status.is_err());
    assert_eq!(
        scan_status.unwrap_err().0,
        "Symbol is not allowed right after questionmark"
    );
}

#[test]
fn ensure_not_number_after_question() {
    let (_, scan_status) = scan_tokens(&String::from("Int?123"));
    assert!(scan_status.is_err());
    assert_eq!(
        scan_status.unwrap_err().0,
        "Symbol is not allowed right after questionmark"
    );
}


#[test]
fn ensure_no_double_questionmark() {
    let (_, scan_status) = scan_tokens(&String::from("Int??"));
    assert!(scan_status.is_err());
    assert_eq!(
        scan_status.unwrap_err().0,
        "Double-question mark has no sense (nillable is either ON or OFF)"
    );
}
