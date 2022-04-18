#[rustfmt::skip]
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Token {
    LeftParenthesis, RightParenthesis,
    LeftCurlyBrackets, RightCurlyBrackets,
    LeftSquareBrackets, RightSquareBrackets,

    Comma, Dot, Semicolon,
    Plus, Minus, Star, Slash,
    Bang, BangEqual, 
    Greater, GreaterEqual,
    Less, LessEqual,
    Equal, EqualEqual,
    Question,
    
    Identifier(String),
    OwnIdentifier(String),
    TypeIdentifier(String),
    String(String),
    Float(f64),
    Integer(i64),

    // Keywords
    Active, Class, Spawn,
    If, Else, Elif,
    While, Foreach, Break, Continue, In,
    Fun,
    From, Import,  // TODO: add "import as" -> as keyword
    True, False, Nil, And, Or, Not,
    Void, This, Return,
    // Let // TODO: implement when type inference is there, should be easy?

    EOF
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScannedToken {
    pub token: Token,
    pub first: usize,
    pub last: usize,
}
pub type ScanningError = (&'static str, usize);

struct Scanner {
    chars: Vec<char>,
    tokens: Vec<ScannedToken>,
    position: usize,
}

impl Scanner {
    fn create(chars: Vec<char>) -> Scanner {
        Scanner { chars, tokens: Vec::new(), position: 0 }
    }

    fn consume_char(&mut self) -> char {
        // returns char and moves position forward
        if !self.is_finished() {
            self.position += 1;
            self.chars[self.position - 1]
        } else {
            '\0'
        }
    }

    fn char_ahead(&self, ahead: usize) -> char {
        // returns char ahead of current position without moving position
        self.chars.get(self.position + ahead).unwrap_or(&'\0').clone()
    }

    fn check_ahead(&self, ahead: usize, expected: char) -> bool {
        self.char_ahead(ahead) == expected
    }

    fn check_next(&mut self, expected: char) -> bool {
        let is_equal_next = self.check_ahead(0, expected);
        if is_equal_next {
            self.consume_char();
        }

        is_equal_next
    }

    fn is_finished(&self) -> bool {
        self.position == self.chars.len()
    }

    fn add_token(&mut self, token: Token) {
        self.tokens
            .push(ScannedToken { token, first: self.position - 1, last: self.position - 1 });
    }

    fn add_token_with_position(&mut self, token: Token, start: usize) {
        self.tokens
            .push(ScannedToken { token, first: start, last: self.position - 1 });
    }
}

fn identifier_to_token(s: String) -> Token {
    match s.as_str() {
        _ if s.chars().next().unwrap().is_uppercase() => Token::TypeIdentifier(s),
        "active" => Token::Active,
        "class" => Token::Class,
        "spawn" => Token::Spawn,
        "if" => Token::If,
        "else" => Token::Else,
        "elif" => Token::Elif,
        "while" => Token::While,
        "foreach" => Token::Foreach,
        "break" => Token::Break,
        "continue" => Token::Continue,
        "in" => Token::In,
        "fun" => Token::Fun,
        "from" => Token::From,
        "import" => Token::Import,
        "true" => Token::True,
        "false" => Token::False,
        "nil" => Token::Nil,
        "void" => Token::Void,
        "and" => Token::And,
        "or" => Token::Or,
        "not" => Token::Not,
        "this" => Token::This,
        // "caller" => Token::Caller,
        "return" => Token::Return,
        _ => Token::Identifier(s),
    }
}

fn scan_string(scanner: &mut Scanner, start: usize, quote: char) -> Option<ScanningError> {
    while !(scanner.is_finished() || scanner.check_ahead(0, quote)) {
        scanner.consume_char();
        if scanner.char_ahead(0) == '\n' {
            return Some(("String must be terminated at the same newline!", start));
        }
    }
    if scanner.is_finished() {
        return Some(("String is not terminated!", start));
    } else {
        let content: String = scanner.chars[start + 1..scanner.position].iter().collect();
        scanner.consume_char();
        scanner.add_token_with_position(Token::String(content), start);
    }

    None
}

fn scan_identifier(scanner: &mut Scanner, start: usize) -> Token {
    while !scanner.is_finished() {
        let c = scanner.char_ahead(0);
        if c.is_alphanumeric() || c == '_' {
            scanner.consume_char();
        } else {
            break;
        }
    }

    let s: String = scanner.chars[start..scanner.position].iter().collect();
    identifier_to_token(s)
}

pub fn scan_tokens(data: &str) -> Result<Vec<ScannedToken>, ScanningError> {
    let mut scanner = Scanner::create(data.chars().collect::<Vec<_>>());

    while !scanner.is_finished() {
        let start = scanner.position;
        match scanner.consume_char() {
            '(' => scanner.add_token(Token::LeftParenthesis),
            ')' => scanner.add_token(Token::RightParenthesis),
            '[' => scanner.add_token(Token::LeftSquareBrackets),
            ']' => scanner.add_token(Token::RightSquareBrackets),
            '{' => scanner.add_token(Token::LeftCurlyBrackets),
            '}' => scanner.add_token(Token::RightCurlyBrackets),

            ',' => scanner.add_token(Token::Comma),
            '.' => scanner.add_token(Token::Dot),
            ';' => scanner.add_token(Token::Semicolon),
            '+' => scanner.add_token(Token::Plus),
            '-' => scanner.add_token(Token::Minus),
            '*' => scanner.add_token(Token::Star),
            '/' => {
                if scanner.check_next('/') {
                    // comment found, skip everything until newline
                    while !scanner.is_finished() && !scanner.check_ahead(0, '\n') {
                        scanner.consume_char();
                    }

                    // Consume trailing \n
                    scanner.consume_char();
                } else if scanner.check_next('*') {
                    let is_commend_end =
                        |s: &Scanner| s.check_ahead(0, '*') && s.check_ahead(1, '/');
                    while !scanner.is_finished() && !is_commend_end(&scanner) {
                        scanner.consume_char();
                    }

                    // Consume both Start and Slash
                    scanner.consume_char();
                    scanner.consume_char();
                } else {
                    scanner.add_token(Token::Slash)
                }
            }
            '?' => {
                let next_char = scanner.char_ahead(0);
                if next_char == '?' {
                    return Err((
                        "Double-question mark has no sense (nillable is either ON or OFF)",
                        scanner.position - 1,
                    ));
                }
                if !(next_char.is_whitespace()
                    || next_char == ','
                    || next_char == ']'
                    || next_char == ')'
                    || next_char == '\0')
                {
                    return Err((
                        "Symbol is not allowed right after questionmark",
                        scanner.position - 1,
                    ));
                }
                scanner.add_token(Token::Question)
            }
            '@' => {
                if !scanner.char_ahead(0).is_alphabetic() {
                    return Err(("Identifier required after @", scanner.position));
                }
                let token = scan_identifier(&mut scanner, start + 1);
                match token {
                    Token::Identifier(s) => {
                        scanner.add_token_with_position(Token::OwnIdentifier(s), start)
                    }
                    _ => return Err(("Identifier required after @", scanner.position)),
                }
            }

            '=' if scanner.check_next('=') => scanner.add_token(Token::EqualEqual),
            '=' => scanner.add_token(Token::Equal),

            '<' if scanner.check_next('=') => scanner.add_token(Token::LessEqual),
            '<' => scanner.add_token(Token::Less),

            '>' if scanner.check_next('=') => scanner.add_token(Token::GreaterEqual),
            '>' => scanner.add_token(Token::Greater),

            '!' if scanner.check_next('=') => scanner.add_token(Token::BangEqual),
            '!' => scanner.add_token(Token::Bang),
            // TODO: think about <=! for send-and-wait pattern
            '"' => {
                let scan_res = scan_string(&mut scanner, start, '"');
                if scan_res.is_some() {
                    return Err(scan_res.unwrap());
                }
            }
            '\'' => {
                let scan_res = scan_string(&mut scanner, start, '\'');
                if scan_res.is_some() {
                    return Err(scan_res.unwrap());
                }
            }

            d if d.is_digit(10) => {
                let mut is_float = false;
                while scanner.char_ahead(0).is_digit(10) {
                    scanner.consume_char();
                }
                if scanner.check_ahead(0, '.') && scanner.char_ahead(1).is_digit(10) {
                    is_float = true;
                    scanner.consume_char();
                    while scanner.char_ahead(0).is_digit(10) {
                        scanner.consume_char();
                    }
                }

                let content: String = scanner.chars[start..scanner.position].iter().collect();
                let token = match is_float {
                    true => Token::Float(content.parse().unwrap()),
                    _ => Token::Integer(content.parse().unwrap()),
                };
                scanner.add_token_with_position(token, start);
            }

            c if c.is_alphabetic() => {
                let token = scan_identifier(&mut scanner, start.clone());

                scanner.add_token_with_position(token, start);
            }
            c if c.is_whitespace() => (),

            _ => {
                return Err(("Unknown symbol occured", scanner.position - 1));
            }
        }
    }
    scanner.add_token_with_position(Token::EOF, data.len() - 1);

    Ok(scanner.tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scan_tokens_helper(s: &str) -> Vec<Token> {
        let res = scan_tokens(s);
        assert!(res.is_ok(), "Error on scanning: {:?}", res.unwrap_err());
        let res = res.unwrap();

        let mut tokens = res.iter().map(|t| t.token.clone()).collect::<Vec<Token>>();
        assert_eq!(tokens.last().unwrap(), &Token::EOF);

        tokens.truncate(tokens.len() - 1);
        tokens
    }

    #[test]
    fn test_positions() {
        let test_string = r#" 123 . [] "hey" 888.888 "#;
        let res = scan_tokens(test_string);
        
        assert_eq!(
            res.unwrap(),
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

        assert!(scan_tokens("@").is_err());
        assert!(scan_tokens("@ field").is_err());
        assert!(scan_tokens("(@)field").is_err());
        assert!(scan_tokens("@.field").is_err());
        assert!(scan_tokens("@2field").is_err());
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
    fn ensure_not_alpha_after_question() {
        let res = scan_tokens(&String::from("Int?asd"));
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().0,
            "Symbol is not allowed right after questionmark"
        );
    }

    #[test]
    fn ensure_not_number_after_question() {
        let res = scan_tokens(&String::from("Int?123"));
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().0,
            "Symbol is not allowed right after questionmark"
        );
    }

    #[test]
    fn ensure_not_dot_after_question() {
        let res = scan_tokens(&String::from("Int?."));
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().0,
            "Symbol is not allowed right after questionmark"
        );
    }

    #[test]
    fn ensure_no_double_questionmark() {
        let res = scan_tokens(&String::from("Int??"));
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().0,
            "Double-question mark has no sense (nillable is either ON or OFF)"
        );
    }
}
