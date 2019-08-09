use std::io::prelude::*;
use std::io::BufReader;
use std::io::Result;
use std::io::Write;
use std::fs::File;

mod scanner;
mod vm;

use scanner::{Line, Token, TokenType};
use vm::{Chunk, InterpretResult, OpCode, Value};

type SourceCode = Vec<char>;

fn report_error(error_token: &Token, source: &SourceCode, message: &str) {
    println!("Error at {} (line {}) : {}",
             error_token.get_token(source),
             error_token.line,
             message);
}

fn emit_byte(chunk: &mut Chunk, op_code: OpCode, line: Line) {
    chunk.write_code(op_code, line);
}

fn advance(tokens: &Vec<Token>, offset: &mut usize) {
    if *offset < tokens.len() {
       *offset += 1;
    }
}

fn function(token: &Token, chunk: &mut Chunk, source: &SourceCode) {
    let name = token.get_token(source);
    match name.as_str() {
        "+" => chunk.write_code(OpCode::Add, token.line),
        "-" => chunk.write_code(OpCode::Subtract, token.line),
        "*" => chunk.write_code(OpCode::Multiply, token.line),
        "/" => chunk.write_code(OpCode::Divide, token.line),
        "not" => chunk.write_code(OpCode::Not, token.line),
        "=" => chunk.write_code(OpCode::Equal, token.line),
        ">" => chunk.write_code(OpCode::GreaterThan, token.line),
        ">=" => {
            chunk.write_code(OpCode::LessThan, token.line);
            chunk.write_code(OpCode::Not, token.line);
        }
        "<" => chunk.write_code(OpCode::LessThan, token.line),
        "<=" => {
            chunk.write_code(OpCode::GreaterThan, token.line);
            chunk.write_code(OpCode::Not, token.line);
        }
        "print" => chunk.write_code(OpCode::Print, token.line),
        _ => report_error(token, source, format!("Unsupported function: {}", name).as_str()),
    }
}

fn sexp(tokens: &Vec<Token>, offset: &mut usize, chunk: &mut Chunk, source: &SourceCode) {
    advance(tokens, offset);
    let token = &tokens[*offset];
    if token.token_type == TokenType::Symbol {
        advance(tokens, offset);
        while tokens[*offset].token_type != TokenType::CloseParenthesis {
            // TODO count number of expressions and pop this many as arguments
            expression(tokens, offset, chunk, source);
        }
        function(token, chunk, source);
        consume_token(tokens, offset, chunk, &TokenType::CloseParenthesis, source);
    } else {
        report_error(token, source, "Function name must be a symbol")
    }
}

fn expression(tokens: &Vec<Token>, offset: &mut usize, chunk: &mut Chunk, source: &SourceCode) {
    let token = &tokens[*offset];
    match token.token_type {
        TokenType::OpenParenthesis => sexp(tokens, offset, chunk, source),
        TokenType::Nil => {
            let idx = chunk.write_constant(Value::Nil);
            chunk.write_code(OpCode::Constant(idx), token.line);
            *offset += 1;
        }
        TokenType::Bool => {
            let val: bool = token.get_token(source) == "true";
            let idx = chunk.write_constant(Value::Bool(val));
            chunk.write_code(OpCode::Constant(idx), token.line);
            *offset += 1;
        }
        TokenType::Int => {
            let val: i64 = token.get_token(source).parse().unwrap();
            let idx = chunk.write_constant(Value::Int(val));
            chunk.write_code(OpCode::Constant(idx), token.line);
            *offset += 1;
        }
        TokenType::Float => {
            let val: f64 = token.get_token(source).parse().unwrap();
            let idx = chunk.write_constant(Value::Float(val));
            chunk.write_code(OpCode::Constant(idx), token.line);
            *offset += 1;
        }
        TokenType::Keyword => {
            println!("parsed a keyword: {}", token.get_token(source));
            *offset += 1;
        }
        TokenType::String => {
            let val = token.get_token(source);
            let idx = chunk.write_constant(Value::String(val));
            chunk.write_code(OpCode::Constant(idx), token.line);
            *offset += 1;
        }
        TokenType::Symbol => {
            println!("parsed a symbol: {}", token.get_token(source));
            *offset += 1;
        }
        TokenType::EOF => {
            *offset += 1;
        }
        _ => {
            report_error(&token, source, "Token type not implemented");
            *offset += 1;
        }
    };
}

fn consume_token(tokens: &Vec<Token>, offset: &mut usize, _chunk: &mut Chunk,
                 expected_type: &TokenType, source: &SourceCode) {
    let token = &tokens[*offset];
    if token.token_type == *expected_type {
        *offset += 1;
    } else {
        report_error(&token, source, "Did not find expected token type");
    };
}

fn compile(source: String) -> Option<Chunk> {
    let source_chars: SourceCode = source.chars().collect();
    let tokens = scanner::scan(&source_chars, false);
    let mut chunk = Chunk{
        code: vec![],
        constants: vec![],
        lines: vec![],
    };
    let mut panic_mode = false;
    let mut had_error = false;
    let mut offset = 0;
    let token_count = tokens.len();
    while offset < token_count {
        let token = &tokens[offset];
        if token.is_error() && !panic_mode {
            report_error(&token, &source_chars, "Lexing error");
            panic_mode = true;
            had_error = true;
        }
        expression(&tokens, &mut offset, &mut chunk, &source_chars);
    }
    emit_byte(&mut chunk, OpCode::Return, 99);
    if had_error {
        None
    } else {
        Some(chunk)
    }
}

fn interpret<'a>(source: String) -> InterpretResult<'a> {
    match compile(source) {
        None => vm::InterpretResult::CompileError,
        Some(byte_code) => vm::init_vm(byte_code).interpret(true)
    }
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
        match interpret(input) {
            InterpretResult::CompileError => println!("Compile error"),
            InterpretResult::RuntimeError(msg) => println!("{}", msg),
            _ => (),
        }
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
        vm::InterpretResult::RuntimeError(_) => std::process::exit(70),
    }
}

fn main() -> Result<()> {
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
