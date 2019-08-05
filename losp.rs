use std::io::prelude::*;
use std::io::BufReader;
use std::io::Result;
use std::io::Write;
use std::fs::File;

mod scanner;
mod vm;

use scanner::{Token, TokenType};
use vm::{Chunk, InterpretResult, OpCode, Value};

fn report_error(error_token: &Token, source: &Vec<char>) {
    println!("Error at {}: {:?}",
             error_token.get_token(source),
             error_token);
}

fn emit_byte(chunk: &mut Chunk, op_code: OpCode, line: u32) {
    chunk.write_code(op_code, line);
}

fn consume_token(token: &Token, expected_type: &TokenType, source: &Vec<char>) {
    if token.token_type == *expected_type {
        ()
    } else {
        report_error(token, source);
    };
}

fn compile(source: String) -> Option<Chunk> {
    let source_chars: Vec<char> = source.chars().collect();
    let tokens = scanner::scan(&source_chars, false);
    let mut chunk = Chunk{
        code: vec![],
        constants: vec![],
        lines: vec![],
    };
    let mut panic_mode = false;
    let mut had_error = false;
    for token in tokens {
        if token.is_error() && !panic_mode {
            report_error(&token, &source_chars);
            panic_mode = true;
            had_error = true;
        }
        consume_token(&token, &token.token_type, &source_chars);
    };
    chunk.write_constant(Value::Float(1.2));
    emit_byte(&mut chunk, OpCode::Constant(0), 1);
    emit_byte(&mut chunk, OpCode::Return, 2);
    if had_error {
        None
    } else {
        Some(chunk)
    }
}

fn interpret(source: String) -> InterpretResult {
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
