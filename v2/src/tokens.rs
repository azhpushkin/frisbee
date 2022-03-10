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
    // let mut line: i32 = 0;

    let mut check_next_char = |cur: &usize, c: char| data_vec.get(cur+1).eq(&Some(&c));

    while current < data_vec.len()  {
        match data_vec[current] {
            '(' => tokens.push((Token::LeftParenthesis, 0)),
            ')' => tokens.push((Token::RightParenthesis, 0)),
            '[' => tokens.push((Token::LeftSquareBrackets, 0)),
            ']' => tokens.push((Token::RightSquareBrackets, 0)),
            '{' => tokens.push((Token::LeftCurlyBrackets, 0)),
            '}' => tokens.push((Token::RightCurlyBrackets, 0)),

            ',' => tokens.push((Token::Comma, 0)),
            '.' => tokens.push((Token::Dot, 0)),
            ';' => tokens.push((Token::Semicolon, 0)),
            '+' => tokens.push((Token::Plus, 0)),
            '-' => tokens.push((Token::LeftParenthesis, 0)),
            '*' => tokens.push((Token::LeftParenthesis, 0)),
            // No slash due to comments

            '<' if check_next_char(&current, '=') => tokens.push((Token::LessEqual, 0)),
            '<' => tokens.push((Token::Less, 0)),
            '>' if check_next_char(&current, '=') => tokens.push((Token::GreaterEqual, 0)),
            '>' => tokens.push((Token::Greater, 0)),
            '!' if check_next_char(&current, '=') => tokens.push((Token::BangEqual, 0)),
            '!' => tokens.push((Token::Bang, 0)),
            '=' if check_next_char(&current, '=') => tokens.push((Token::EqualEqual, 0)),
            '=' => tokens.push((Token::Equal, 0)),

            // '"' => {
            //     let mut off: usize = 1;
            //     while data_vec[current+off] != '"' { off += 1; println!("1")}
            //     if data_vec[current+off] != '"' {
            //         panic!("LOL");
            //     } else {
            //         tokens.push((Token::String(data_vec[current..current+off].into_iter().collect()));
            //     }

            // },

            _ => {}
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
        assert_eq!(res[0], (Token::LeftParenthesis, 0));
        assert_eq!(res[1], (Token::RightParenthesis, 0));
        assert_eq!(res[2], (Token::LeftSquareBrackets, 0));
        assert_eq!(res[3], (Token::RightSquareBrackets, 0));
        assert_eq!(res[4], (Token::Dot, 0));
        assert_eq!(res[5], (Token::Comma, 0));
    }
}