use strum_macros::Display;

#[rustfmt::skip]
#[derive(Display, Debug, PartialEq, PartialOrd, Clone)]
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
    
    Identifier(String),
    TypeIdentifier(String),
    String(String),
    Float(f32),
    Integer(i32),

    // Keywords
    Active, Passive, Spawn, New,
    If, Else, Elif, While, For, // todo; break, continue
    Let, Def,
    From, Import,
    True, False, Nil, And, Or, Not,
    This, Caller, Return,

    EOF
}

pub type ScannedToken = (Token, i32);

struct Scanner {
    chars: Vec<char>,
    tokens: Vec<ScannedToken>,
    position: usize,
}

impl Scanner {
    fn create(chars: Vec<char>) -> Scanner {
        Scanner {
            chars,
            tokens: Vec::new(),
            position: 0,
        }
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
        self.chars
            .get(self.position + ahead)
            .unwrap_or(&'\0')
            .clone()
    }

    fn check_ahead(&self, ahead: usize, expected: char) -> bool {
        self.char_ahead(ahead) == expected
    }

    fn is_finished(&self) -> bool {
        self.position == self.chars.len()
    }

    fn add_token(&mut self, token: Token) {
        self.tokens.push((token, (self.position - 1) as i32))
    }

    fn add_token_with_position(&mut self, token: Token, pos: usize) {
        self.tokens.push((token, pos as i32))
    }
}

fn identifier_to_token(s: String) -> Token {
    match s.as_str() {
        _ if s.chars().next().unwrap().is_uppercase() => Token::TypeIdentifier(s),
        "active" => Token::Active,
        "passive" => Token::Passive,
        "spawn" => Token::Spawn,
        "new" => Token::New,
        "if" => Token::If,
        "else" => Token::Else,
        "elif" => Token::Elif,
        "while" => Token::While,
        "for" => Token::For,
        "let" => Token::Let,
        "def" => Token::Def,
        "from" => Token::From,
        "import" => Token::Import,
        "true" => Token::True,
        "false" => Token::False,
        "nil" => Token::Nil,
        "and" => Token::And,
        "or" => Token::Or,
        "not" => Token::Not,
        "this" => Token::This,
        "caller" => Token::Caller,
        "return" => Token::Return,
        _ => Token::Identifier(s),
    }
}

pub fn scan_tokens(data: String) -> Vec<ScannedToken> {
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
                if scanner.check_ahead(0, '/') {
                    scanner.consume_char();
                    while !scanner.is_finished() && scanner.char_ahead(0) != '\n' {
                        scanner.consume_char();
                    }
                } else {
                    scanner.add_token(Token::Slash)
                }
            }

            '=' if scanner.check_ahead(0, '=') => {
                scanner.consume_char();
                scanner.add_token(Token::EqualEqual)
            }
            '<' if scanner.check_ahead(0, '=') => {
                scanner.consume_char();
                scanner.add_token(Token::LessEqual)
            }
            '>' if scanner.check_ahead(0, '=') => {
                scanner.consume_char();
                scanner.add_token(Token::GreaterEqual)
            }
            '!' if scanner.check_ahead(0, '=') => {
                scanner.consume_char();
                scanner.add_token(Token::BangEqual)
            }

            '=' => scanner.add_token(Token::Equal),
            '<' => scanner.add_token(Token::Less),
            '>' => scanner.add_token(Token::Greater),
            '!' => scanner.add_token(Token::Bang),

            '"' => {
                while !(scanner.is_finished() || scanner.check_ahead(0, '"')) {
                    scanner.consume_char();
                }
                if scanner.is_finished() {
                    panic!("String is not terminated!");
                } else {
                    let content: String =
                        scanner.chars[start + 1..scanner.position].iter().collect();
                    scanner.add_token_with_position(Token::String(content), start);
                    scanner.consume_char();
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
                if is_float {
                    let num: f32 = content.parse().unwrap();
                    scanner.add_token_with_position(Token::Float(num), start);
                } else {
                    let num: i32 = content.parse().unwrap();
                    scanner.add_token_with_position(Token::Integer(num), start);
                }
            }

            c if c.is_alphabetic() => {
                while !scanner.is_finished() {
                    let c = scanner.char_ahead(0);
                    if c.is_alphanumeric() || c == '_' {
                        scanner.consume_char();
                    } else {
                        break;
                    }
                }

                let s: String = scanner.chars[start..scanner.position].iter().collect();

                scanner.add_token_with_position(identifier_to_token(s), start);
            }
            ' ' => (),
            '\t' => (),
            '\n' => (),

            c => {
                panic!("Unknown symbol occured: {}", c);
            }
        }
    }
    scanner.add_token_with_position(Token::EOF, data.len() - 1);

    scanner.tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scan_tokens_helper(s: &str) -> Vec<Token> {
        let res = scan_tokens(String::from(s));

        let mut tokens = res.iter().map(|(t, _p)| t.clone()).collect::<Vec<Token>>();
        assert_eq!(tokens.last().unwrap(), &Token::EOF);

        tokens.truncate(tokens.len() - 1);
        tokens
    }

    #[test]
    fn test_positions() {
        let res = scan_tokens(String::from(r#" 123 . [] "hey" 888.888 "#));
        assert_eq!(
            res,
            vec![
                (Token::Integer(123), 1),
                (Token::Dot, 5),
                (Token::LeftSquareBrackets, 7),
                (Token::RightSquareBrackets, 8),
                (Token::String(String::from("hey")), 10),
                (Token::Float(888.888), 16),
                (Token::EOF, 23),
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
            scan_tokens_helper("+ - / * "),
            vec![Token::Plus, Token::Minus, Token::Slash, Token::Star]
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
    fn test_keywords() {
        assert_eq!(
            scan_tokens_helper("if else spawn active passive"),
            vec![Token::If, Token::Else, Token::Spawn, Token::Active, Token::Passive,]
        );
    }
}
