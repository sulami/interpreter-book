#[derive(Debug,PartialEq)]
pub enum ScanError {
    UnterminatedString,
    EmptyKeyword,
    RanOff,
}

#[derive(Debug,PartialEq)]
pub enum TokenType {
    // parens
    OpenParenthesis, CloseParenthesis,
    OpenBracket, CloseBracket,
    OpenBrace, CloseBrace,
    // literals
    Nil, Bool, Int, Float, String,
    // special syntax
    Quote,
    // keywords
    Keyword,
    // symbols
    Symbol,
    // we're done here
    EOF,
    // i am
    Error(ScanError),
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

pub type Line = u32;

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub line: Line,
    start: usize,
    length: usize,
}

impl Token {
    pub fn is_error(&self) -> bool {
        match self.token_type {
            TokenType::Error(_) => true,
            _ => false,
        }
    }

    pub fn get_token(&self, source: &Vec<char>) -> String {
        match self.token_type {
            TokenType::String => source[self.start+1..self.start+self.length - 1]
                .into_iter()
                .collect(),
            _ => source[self.start..self.start+self.length]
                .into_iter()
                .collect(),
        }
    }
}

fn is_number(c: char) -> bool {
    c.is_ascii_digit()
        || c == '.'
}

fn is_symbol(c: char) -> bool {
    c.is_alphanumeric()
        || c == '-'
        || c == '_'
        || c == '>'
        || c == '<'
        || c == '*'
        || c == '+'
        || c == '.'
        || c == '!'
        || c == '?'
        || c == '/'
        || c == ':'
        || c == '='
}

fn advance(source: &Vec<char>, offset: &mut usize, line: &mut Line) -> bool {
    if *offset == source.len() - 1 {
        return false
    }
    if source[*offset] == '\n' {
        *line += 1;
    }
    *offset += 1;
    true
}

fn peek(source: &Vec<char>, offset: usize) -> Option<char> {
    if offset == source.len() {
        None
    } else {
        Some(source[offset + 1])
    }
}

fn skip_whitespace(source: &Vec<char>, start: &mut usize, line: &mut Line) {
    while *start < source.len() - 1 && source[*start].is_whitespace() {
        advance(source, start, line);
    }
}

fn skip_comments(source: &Vec<char>, start: &mut usize, line: &mut Line) {
    if source[*start] == ';' {
        while *start < source.len() - 1 && source[*start] != '\n' {
            advance(source, start, line);
        }
        advance(source, start, line); // skip the newline
    }
}

fn skip_non_code(source: &Vec<char>, start: &mut usize, line: &mut Line) {
    loop {
        let started_at = *start;
        skip_whitespace(source, start, line);
        skip_comments(source, start, line);
        if *start == started_at {
            break
        }
    }
}

fn scan_string(source: &Vec<char>, start: &mut usize, line: &mut Line) -> (TokenType, usize) {
    let mut string_end = *start;
    loop {
        advance(source, &mut string_end, line);
        let string_length = string_end - *start;
        if source[string_end] == '"' {
            break (TokenType::String, string_length + 1)
        }
        if source.len() <= string_end {
            break (TokenType::Error(ScanError::UnterminatedString), string_length)
        }
    }
}

fn scan_keyword(source: &Vec<char>, start: &mut usize, line: &mut Line) -> (TokenType, usize) {
    let mut keyword_end = *start;
    while is_symbol(source[keyword_end]) {
        advance(source, &mut keyword_end, line);
    }
    let keyword_length = keyword_end - *start;
    if keyword_length == 1 {
        (TokenType::Error(ScanError::EmptyKeyword), keyword_length)
    } else {
        (TokenType::Keyword, keyword_length)
    }
}

fn scan_number(source: &Vec<char>, start: &mut usize) -> (TokenType, usize) {
    let mut token_length = 1;
    let mut is_float = false;
    loop {
        if source.len() <= *start + token_length {
            break
        };
        let c = source[*start + token_length];
        if !is_number(c) {
            break
        }
        if c == '.' {
            is_float = true;
        }
        token_length += 1;
    }
    if 1 == token_length && '-' == source[*start + token_length] {
        (TokenType::Symbol, token_length)
    } else if is_float {
        (TokenType::Float, token_length)
    } else {
        (TokenType::Int, token_length)
    }
}

fn scan_symbol(source: &Vec<char>, start: &mut usize, line: &mut Line) -> (TokenType, usize) {
    let mut symbol_end = *start;
    while advance(source, &mut symbol_end, line) {
        if !is_symbol(source[symbol_end]) {
            break
        }
    }
    let token_length = symbol_end - *start;
    match source[*start..symbol_end] {
        ['n','i','l']=> (TokenType::Nil, token_length),
        ['t','r','u','e']=> (TokenType::Bool, token_length),
        ['f','a','l','s','e']=> (TokenType::Bool, token_length),
        _ => (TokenType::Symbol, token_length),
    }
}

fn scan_dash(source: &Vec<char>, start: &mut usize, line: &mut Line) -> (TokenType, usize) {
    let next_char = peek(source, *start);
    match next_char {
        None => (TokenType::Symbol, 1),
        Some(c) => {
            if c.is_whitespace() {
                (TokenType::Symbol, 1)
            } else if c.is_numeric() {
                scan_number(source, start)
            } else {
                scan_symbol(source, start, line)
            }
        }
    }
}

fn scan_token(source: &Vec<char>, offset: usize, line: &mut Line) -> Token {
    let mut start = offset;
    skip_non_code(source, &mut start, line);
    let (token_type, length) = match source[start] {
        '(' => (TokenType::OpenParenthesis, 1),
        ')' => (TokenType::CloseParenthesis, 1),
        '[' => (TokenType::OpenBracket, 1),
        ']' => (TokenType::CloseBracket, 1),
        '{' => (TokenType::OpenBrace, 1),
        '}' => (TokenType::CloseBrace, 1),
        '\'' => (TokenType::Quote, 1),
        '"' => scan_string(source, &mut start, line),
        ':' => scan_keyword(source, &mut start, line),
        '-' => scan_dash(source, &mut start, line),
        _ if start == source.len() - 1 => (TokenType::EOF, 0),
        _ if is_number(source[start]) => scan_number(source, &mut start),
        _ if is_symbol(source[start]) => scan_symbol(source, &mut start, line),
        _ => (TokenType::Error(ScanError::RanOff), 1),
    };
    Token {
        token_type: token_type,
        line: *line,
        start: start,
        length: length,
    }
}

fn print_token(token: &Token, source: &Vec<char>) {
    println!("{:?} {} {} {} {}",
             token.token_type,
             token.line,
             token.length,
             token.start,
             token.get_token(&source));
}

pub fn scan(source: &Vec<char>, debug: bool) -> Vec<Token> {
    let mut offset = 0;
    let mut tokens: Vec<Token> = vec![];
    let mut line: Line = 1;
    loop {
        let token = scan_token(&source, offset, &mut line);
        if debug {
            print_token(&token, source);
        }
        offset = token.start + token.length;
        if source.len() <= offset {
            tokens.append(&mut vec![token]);
            let eof_token = Token{token_type: TokenType::EOF, line: line, start: offset, length: 0};
            if debug {
                print_token(&eof_token, source);
            }
            tokens.append(
                &mut vec![eof_token]
            );
            break tokens;
        } else if token.token_type == TokenType::EOF {
            tokens.append(&mut vec![token]);
            break tokens;
        } else {
            tokens.append(&mut vec![token]);
        }
    }
}
