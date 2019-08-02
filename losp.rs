mod vm;
// use vm::{Chunk, OpCode, Value};

use std::io::prelude::*;
use std::io::BufReader;
use std::io::Result;
use std::io::Write;
use std::fs::File;

#[allow(dead_code)]
#[derive(Debug,PartialEq)]
enum TokenType {
    OpenParenthesis,
    CloseParenthesis,
    Symbol,
    EOF,
    Error,
}

struct Token {
    token_type: TokenType,
    line: usize,
    start: usize,
    length: usize,
}

fn scan_token(source: &Vec<char>, offset: usize, line: usize) -> Token {
    if offset == source.len() {
        Token {
            token_type: TokenType::EOF,
            start: offset,
            line: line,
            length: 1,
        }
    } else {
        match source[offset] {
            '(' => Token {
                token_type: TokenType::OpenParenthesis,
                start: offset,
                line: line,
                length: 1,
            },
            ')' => Token {
                token_type: TokenType::CloseParenthesis,
                start: offset,
                line: line,
                length: 1,
            },
            _ => Token {
                token_type: TokenType::Symbol,
                start: offset,
                line: line,
                length: 1,
            },
        }
    }
}

fn scan(source: String) -> Vec<Token> {
    let source_chars = source.chars().collect();
    let mut offset = 0;
    let mut line = 0;
    let mut tokens = vec![];
    loop {
        let token = scan_token(&source_chars, offset, line);
        if token.line != line {
            print!("{:4}", token.line);
            line = token.line;
        } else {
            print!("   | ");
        }
        println!("{:?} {} {}", token.token_type, token.length, token.start);
        if token.token_type == TokenType::EOF {
            break tokens;
        }
        offset = offset + token.length;
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
