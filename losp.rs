use std::io::prelude::*;
use std::io::BufReader;
use std::io::Result;
use std::io::Write;
use std::fs::File;

mod compiler;

use compiler::interpret;
use compiler::vm::{InterpretResult, init_vm};

fn repl(debug: bool) -> Result<()> {
    let mut vm = init_vm();
    loop {
        print!("> ");
        let _ = std::io::stdout().flush();
        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input);
        if input == "\n" {
            println!("");
            break;
        }
        match interpret(&mut vm, input, debug) {
            InterpretResult::CompileError => println!("Compile error"),
            InterpretResult::RuntimeError(msg) => println!("{}", msg),
            _ => (),
        }
    }
    Ok(())
}

fn run_file(path: &String, debug: bool) -> Result<()> {
    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(file);
    let mut source = String::new();
    buf_reader.read_to_string(&mut source)?;
    let mut vm = init_vm();
    match interpret(&mut vm, source, debug) {
        InterpretResult::OK => Ok(()),
        InterpretResult::CompileError => std::process::exit(65),
        InterpretResult::RuntimeError(msg) => {
            println!("{}", msg);
            std::process::exit(70);
        }
    }
}

fn main() -> Result<()> {
    let opts = std::env::args();
    match opts.len() {
        1 => repl(false),
        2 => run_file(&opts.last().expect("the world is ending"), false),
        3 => run_file(&opts.last().expect("the world is ending"), true),
        _ => {
            let name = "losp";
            println!("useage:");
            println!("{}        - start repl", name);
            println!("{} <file> - run file", name);
            std::process::exit(64);
        }
    }
}
