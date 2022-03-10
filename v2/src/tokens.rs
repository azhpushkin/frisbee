use std::{default, ptr::null};

use strum_macros::Display;

#[derive(Display, Debug, PartialEq, PartialOrd)]
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
    String(String),
    Float(f32),
    Integer(i32),

    // Keywords
    Active, Passive, Spawn, New,
    If, Else, Elif, While, For,
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
    position: usize
}

impl Scanner {
    fn create(chars: Vec<char>) -> Scanner {
        Scanner { chars, tokens: Vec::new(), position: 0}
    }

    fn consume_char(&mut self) -> char {
        // returns char and moves position forward
        self.position += 1;
        self.chars[self.position-1]
    }

    fn char_ahead(&self, ahead: usize) -> char {
        // returns char ahead of current position without moving position
        self.chars.get(self.position + ahead).unwrap_or(&'\0').clone()
    }

    fn check_ahead(&self, ahead: usize, expected: char) -> bool {
        self.char_ahead(ahead) == expected
    }

    fn is_finished(&self) -> bool {
        self.position == self.chars.len()
    }

    fn add_token(&mut self, token: Token) {
        self.tokens.push((token, (self.position-1) as i32))
    }

    fn add_token_with_position(&mut self, token: Token, pos: usize) {
        self.tokens.push((token, pos as i32))
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
            // TODO: No slash due to comments

            '<' if scanner.check_ahead(1, '=') => scanner.add_token(Token::LessEqual),
            '<' => scanner.add_token(Token::Less),
            '>' if scanner.check_ahead(1, '=') => scanner.add_token(Token::GreaterEqual),
            '>' => scanner.add_token(Token::Greater),
            '!' if scanner.check_ahead(1, '=') => scanner.add_token(Token::BangEqual),
            '!' => scanner.add_token(Token::Bang),
            '=' if scanner.check_ahead(1, '=') => scanner.add_token(Token::EqualEqual),
            '=' => scanner.add_token(Token::Equal),

            '"' => {
                while !(scanner.is_finished() || scanner.check_ahead(0, '"')) {
                    scanner.consume_char();
                }
                if scanner.is_finished() {
                    panic!("String is not terminated!");
                } else {
                    let content: String = scanner.chars[start+1..scanner.position].iter().collect();
                    scanner.add_token_with_position(Token::String(content), start);
                    scanner.consume_char();
                }
            },

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
            

            c => {
                panic!("Unknown symbol occured: {}", c);
            }
        }
    }

    scanner.tokens
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn smoke_check() {
        let res = scan_tokens(String::from("()[].,"));
        assert_eq!(res.len(), 6);
        assert_eq!(res[0], (Token::LeftParenthesis, 0));
        assert_eq!(res[1], (Token::RightParenthesis, 1));
        assert_eq!(res[2], (Token::LeftSquareBrackets, 2));
        assert_eq!(res[3], (Token::RightSquareBrackets, 3));
        assert_eq!(res[4], (Token::Dot, 4));
        assert_eq!(res[5], (Token::Comma, 5));
    }

    #[test]
    fn empty_string() {
        let res = scan_tokens(String::from(r#""""#));
        assert_eq!(res.len(), 1);
        assert_eq!(res[0], (Token::String(String::from("")), 0));

        let res = scan_tokens(String::from(r#"!""."#));
        assert_eq!(res.len(), 3);
        assert_eq!(res[0], (Token::Bang, 0));
        assert_eq!(res[1], (Token::String(String::from("")), 1));
        assert_eq!(res[2], (Token::Dot, 3));
    }
    
    #[test]
    fn simple_string() {
        let res = scan_tokens(String::from(r#""123""#));
        assert_eq!(res.len(), 1);
        assert_eq!(res[0], (Token::String(String::from("123")), 0));

    }

    #[test]
    fn string_and_others() {
        let res = scan_tokens(String::from(r#"+"123"-"#));
        assert_eq!(res.len(), 3);

        assert_eq!(res[0], (Token::Plus, 0));
        assert_eq!(res[1], (Token::String(String::from("123")), 1));
        assert_eq!(res[2], (Token::Minus, 6));

    }

    #[test]
    fn float() {
        let res = scan_tokens(String::from("123.456"));
        assert_eq!(res.len(), 1);
        assert_eq!(res[0], (Token::Float(123.456), 0));
    }

    #[test]
    fn integer() {
        let res = scan_tokens(String::from("123"));
        assert_eq!(res.len(), 1);
        assert_eq!(res[0], (Token::Integer(123), 0));
    }
}