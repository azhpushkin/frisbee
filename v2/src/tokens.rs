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
}

pub type ScannedToken = (Token, i32);


pub fn scan_tokens(data: String) -> Vec<ScannedToken> {
    let data_vec = data.chars().collect::<Vec<_>>();
    let mut tokens: Vec<ScannedToken> = Vec::new();
    let mut current: usize = 0;

    let mut check_next_char = |cur: &usize, c: char| data_vec.get(cur+1).eq(&Some(&c));


    while current < data_vec.len()  {
        match data_vec[current] {
            '(' => tokens.push((Token::LeftParenthesis, current as i32)),
            ')' => tokens.push((Token::RightParenthesis, current as i32)),
            '[' => tokens.push((Token::LeftSquareBrackets, current as i32)),
            ']' => tokens.push((Token::RightSquareBrackets, current as i32)),
            '{' => tokens.push((Token::LeftCurlyBrackets, current as i32)),
            '}' => tokens.push((Token::RightCurlyBrackets, current as i32)),

            ',' => tokens.push((Token::Comma, current as i32)),
            '.' => tokens.push((Token::Dot, current as i32)),
            ';' => tokens.push((Token::Semicolon, current as i32)),
            '+' => tokens.push((Token::Plus, current as i32)),
            '-' => tokens.push((Token::Minus, current as i32)),
            '*' => tokens.push((Token::Star, current as i32)),
            // TODO: No slash due to comments

            '<' if check_next_char(&current, '=') => tokens.push((Token::LessEqual, current as i32)),
            '<' => tokens.push((Token::Less, current as i32)),
            '>' if check_next_char(&current, '=') => tokens.push((Token::GreaterEqual, current as i32)),
            '>' => tokens.push((Token::Greater, current as i32)),
            '!' if check_next_char(&current, '=') => tokens.push((Token::BangEqual, current as i32)),
            '!' => tokens.push((Token::Bang, current as i32)),
            '=' if check_next_char(&current, '=') => tokens.push((Token::EqualEqual, current as i32)),
            '=' => tokens.push((Token::Equal, current as i32)),

            '"' => {
                let mut end: usize = current+1;
                while end < data_vec.len() && data_vec[end] != '"' {
                    end += 1;
                }
                println!("Current {}", current);
                println!("End {}", end);
                if end == data_vec.len() {
                    panic!("String is not terminated!")
                } else {
                    let content: String = data_vec[current+1..end].iter().collect();
                    tokens.push((Token::String(content), current as i32));
                    current = end+1;
                    continue;
                }
            }
            

            c => {
                panic!("Unknown symbol occured: {}", c);
            }
        }
        
        // scan
        current += 1;   
    }

    tokens
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
}