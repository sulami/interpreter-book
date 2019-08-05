use std::io::prelude::*;
use std::io::BufReader;
use std::io::Result;
use std::io::Write;
use std::fs::File;

mod scanner;
mod vm;

use vm::{Chunk, OpCode, Value};

fn report_error(error: &scanner::Token, source: &Vec<char>) {
    println!("Error at {}: {:?}", error.get_token(source), error);
}

fn compile(source: String) -> Option<vm::Chunk> {
    let source_chars: Vec<char> = source.chars().collect();
    let tokens = scanner::scan(&source_chars, true);
    let mut panic_mode = false;
    for token in tokens {
        if token.is_error() && !panic_mode {
            report_error(&token, &source_chars);
            panic_mode = true;
        }
    };
    if !panic_mode {
        println!("Compiled!");
    };
    None
}

fn interpret(source: String) -> vm::InterpretResult {
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
