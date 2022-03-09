use strum_macros::Display;

#[derive(Display, Debug)]
pub enum Token {
    LeftParenthesis, RightParenthesis,
    LeftCurlyBrackets, RightCurlyBrackets,
    LeftSquareBrackets, RightSquareBrackets,

    Comma, Dot, Semicolon,
    Plus, Minus, Star, Slash,
    BangEqual, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,
    Equal, Bang, Question,
    
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

pub fn scan_tokens(data: String) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut current: usize = 0;

    // while current < tokens.len() {
    //     // scan
        
    // }
    tokens.push(Token::Less);

    tokens
}