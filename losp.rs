mod vm;
// use vm::{Chunk, OpCode, Value};

use std::io::prelude::*;
use std::io::BufReader;
use std::io::Result;
use std::io::Write;
use std::fs::File;

#[derive(Debug,PartialEq)]
enum TokenType {
    // parens
    OpenParenthesis, CloseParenthesis,
    OpenBracket, CloseBracket,
    // literals
    String, Number,
    // special syntax
    Quote,
    // symbols
    Symbol,
    // i am
    Error,
}

struct Token {
    token_type: TokenType,
    start: usize,
    length: usize,
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
        || c == '!'
        || c == '/'
        || c == ':'
}

fn scan_token(source: &Vec<char>, offset: usize) -> Token {
    let mut start = offset;
    while start < source.len() - 1 && source[start].is_whitespace() {
        start += 1;
    }
    if source[start] == ';' {
        while start < source.len() - 1 && source[start] != '\n' {
            start += 1;
        }
        start += 1; // skip the newline
    }
    let (token_type, length) = match source[start] {
        '(' => (TokenType::OpenParenthesis, 1),
        ')' => (TokenType::CloseParenthesis, 1),
        '[' => (TokenType::OpenBracket, 1),
        ']' => (TokenType::CloseBracket, 1),
        '\'' => (TokenType::Quote, 1),
        '"' => {
            let mut string_length = 1;
            loop {
                if source[start + string_length] == '"' {
                    break (TokenType::String, string_length + 1)
                }
                if source.len() <= start + string_length {
                    break (TokenType::Error, string_length)
                }
                string_length += 1;
            }
        }
        _ => {
            if is_number(source[start]) {
                let mut token_length = 1;
                while start + token_length < source.len()
                    && is_number(source[start + token_length]) {
                        token_length += 1;
                    }
                (TokenType::Number, token_length)
            } else if is_symbol(source[start]) {
                let mut token_length = 1;
                while start + token_length < source.len()
                    && is_symbol(source[start + token_length]) {
                        token_length += 1;
                    }
                (TokenType::Symbol, token_length)
            } else {
                (TokenType::Error, 1)
            }
        },
    };
    Token {
        token_type: token_type,
        start: start,
        length: length,
    }
}

fn scan(source: String) -> Vec<Token> {
    let source_chars: Vec<char> = source.chars().collect();
    let mut offset = 0;
    let mut tokens = vec![];
    loop {
        if offset >= source_chars.len() {
            break tokens;
        }
        let token = scan_token(&source_chars, offset);
        let v: String = source_chars[token.start..token.start+token.length].into_iter().collect();
        println!("{:?} {} {} {}", token.token_type, token.length, token.start, v);
        offset = token.start + token.length;
        tokens.insert(tokens.len(), token);
    }
}

fn compile (source: String) -> vm::InterpretResult {
    scan(source);
    vm::InterpretResult::OK
}

fn interpret(source: String) -> vm::InterpretResult {
    compile(source)
}

fn repl() -> Result<()> {
    loop {
        print!("> ");
        let _ = std::io::stdout().flush();
        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input);
        if input == "" {
            println!("");
            break;
        }
        interpret(input);
    }
    Ok(())
}

fn run_file(path: &String) -> Result<()> {
    println!("Compiling {}...", path);
    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(file);
    let mut source = String::new();
    buf_reader.read_to_string(&mut source)?;
    match interpret(source) {
        vm::InterpretResult::OK => Ok(()),
        vm::InterpretResult::CompileError => std::process::exit(65),
        vm::InterpretResult::RuntimeError => std::process::exit(70),
    }
}

fn main() -> Result<()> {
    // Negate a constant
    // let chunk: Chunk = Chunk{
    //     code: vec![OpCode::Constant(0), OpCode::Negate, OpCode::Return],
    //     lines: vec![123, 123, 123],
    //     constants: vec![Value::Float(1.2)],
    // };
    // init_vm(chunk).interpret(true);

    // Multiply some constants
    // let chunk: Chunk = Chunk{
    //     code: vec![OpCode::Constant(0),
    //                OpCode::Constant(1),
    //                OpCode::Add,
    //                OpCode::Constant(2),
    //                OpCode::Divide,
    //                OpCode::Negate,
    //                OpCode::Return],
    //     lines: vec![123, 124, 125, 125, 125, 126, 126],
    //     constants: vec![Value::Float(1.2), Value::Float(3.4), Value::Float(5.6)],
    // };
    // vm::init_vm(chunk).interpret(true);

    // TODO need a way to init a vm without a chunk
    let opts = std::env::args();
    match opts.len() {
        1 => repl(),
        2 => run_file(&opts.last().expect("the world is ending")),
        _ => {
            let name = "losp";
            println!("useage:");
            println!("{}        - start repl", name);
            println!("{} <file> - run file", name);
            std::process::exit(64);
        }
    }
}
