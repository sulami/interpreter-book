use std::io::prelude::*;
use std::io::BufReader;
use std::io::Result;
use std::io::Write;
use std::fs::File;

mod scanner;
mod vm;

use scanner::{Line, Token, TokenType};
use vm::{Chunk, InterpretResult, OpCode, Value, VM};

type SourceCode = Vec<char>;

struct LocalVar {
    name: String,
    depth: usize,
}

struct Compiler {
    locals: Vec<LocalVar>,
    scope_depth: usize,
}

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

fn sexp(compiler: &mut Compiler, tokens: &Vec<Token>, offset: &mut usize, chunk: &mut Chunk, source: &SourceCode) {
    advance(tokens, offset);
    let token = &tokens[*offset];
    if token.token_type == TokenType::Symbol {
        let fn_name = token.get_token(source);
        if fn_name.as_str() == "def" {
            // `def` needs to read ahead because the first arg is a raw symbol
            advance(tokens, offset);
            let next_token = &tokens[*offset];
            if next_token.token_type == TokenType::Symbol {
                let sym = next_token.get_token(source);
                advance(tokens, offset);
                expression(compiler, tokens, offset, chunk, source);
                let idx = chunk.write_constant(Value::Symbol(sym));
                chunk.write_code(OpCode::DefineGlobal(idx), token.line);
            } else {
                report_error(next_token, source, "Expected symbol for def")
            }
        } else if fn_name.as_str() == "let" {
            advance(tokens, offset);
            compiler.scope_depth += 1;
            consume_token(tokens, offset, &TokenType::OpenParenthesis, source);
            while &tokens[*offset].token_type == &TokenType::OpenParenthesis {
                advance(tokens, offset);
                // TODO error if not a symbol
                let binding_token = &tokens[*offset];
                let name = binding_token.get_token(source);
                advance(tokens, offset);
                expression(compiler, tokens, offset, chunk, source);
                chunk.write_code(OpCode::DefineLocal(compiler.locals.len()), binding_token.line);
                compiler.locals.append(&mut vec![LocalVar{
                    name: name.to_string(),
                    depth: compiler.scope_depth,
                }]);
                consume_token(tokens, offset, &TokenType::CloseParenthesis, source);
            }
            consume_token(tokens, offset, &TokenType::CloseParenthesis, source);
            expression(compiler, tokens, offset, chunk, source);
            compiler.scope_depth -= 1;
            // FIXME if the `let` returns a value, that will be on the stack last
            loop {
                match compiler.locals.last() {
                    None => break,
                    Some(l) => {
                        if compiler.scope_depth < l.depth {
                            compiler.locals.pop();
                            chunk.write_code(OpCode::Pop, token.line);
                        } else {
                            break
                        }
                    }
                }
            }
        } else {
            advance(tokens, offset);
            while tokens[*offset].token_type != TokenType::CloseParenthesis {
                // TODO count number of expressions and pop this many as arguments
                expression(compiler, tokens, offset, chunk, source);
            }
            match fn_name.as_str() {
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
                _ => report_error(token, source, format!("Unsupported function: {}", fn_name).as_str()),
            }
        }
        consume_token(tokens, offset, &TokenType::CloseParenthesis, source);
    } else {
        report_error(token, source, "Function name must be a symbol")
    }
}

fn expression(compiler: &mut Compiler,
              tokens: &Vec<Token>,
              offset: &mut usize,
              chunk: &mut Chunk,
              source: &SourceCode) {
    let token = &tokens[*offset];
    match token.token_type {
        TokenType::OpenParenthesis => sexp(compiler, tokens, offset, chunk, source),
        TokenType::Nil => {
            let idx = chunk.write_constant(Value::Nil);
            chunk.write_code(OpCode::Constant(idx), token.line);
            advance(tokens, offset);
        }
        TokenType::Bool => {
            let val: bool = token.get_token(source) == "true";
            let idx = chunk.write_constant(Value::Bool(val));
            chunk.write_code(OpCode::Constant(idx), token.line);
            advance(tokens, offset);
        }
        TokenType::Int => {
            let val: i64 = token.get_token(source).parse().unwrap();
            let idx = chunk.write_constant(Value::Int(val));
            chunk.write_code(OpCode::Constant(idx), token.line);
            advance(tokens, offset);
        }
        TokenType::Float => {
            let val: f64 = token.get_token(source).parse().unwrap();
            let idx = chunk.write_constant(Value::Float(val));
            chunk.write_code(OpCode::Constant(idx), token.line);
            advance(tokens, offset);
        }
        TokenType::Keyword => {
            println!("parsed a keyword: {}", token.get_token(source));
            advance(tokens, offset);
        }
        TokenType::String => {
            let val = token.get_token(source);
            let idx = chunk.write_constant(Value::String(val));
            chunk.write_code(OpCode::Constant(idx), token.line);
            advance(tokens, offset);
        }
        TokenType::Symbol => {
            let val = token.get_token(source);
            let local_count = compiler.locals.len();
            let mut is_local = false;
            for i in 0..local_count {
                if compiler.locals[local_count - i - 1].name == val {
                    chunk.write_code(OpCode::GetLocal(i), token.line);
                    is_local = true;
                    break
                }
            }
            if !is_local {
                let idx = chunk.write_constant(Value::Symbol(val));
                chunk.write_code(OpCode::GetGlobal(idx), token.line);
            }
            advance(tokens, offset);
        }
        TokenType::EOF => {
            advance(tokens, offset);
        }
        _ => {
            report_error(&token, source, "Token type not implemented");
            advance(tokens, offset);
        }
    };
}

fn consume_token(tokens: &Vec<Token>, offset: &mut usize,
                 expected_type: &TokenType, source: &SourceCode) {
    let token = &tokens[*offset];
    if token.token_type == *expected_type {
        advance(tokens, offset);
    } else {
        report_error(&token, source, "Did not find expected token type");
    };
}

fn compile(source: String) -> Option<Chunk> {
    let mut compiler = Compiler{locals: vec![], scope_depth: 0};
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
        if panic_mode {
            if token.token_type == TokenType::CloseParenthesis
                || token.token_type == TokenType::OpenParenthesis
                || token.token_type == TokenType::EOF {
                    panic_mode = false;
            }
            // TODO this doesn't really sync up yet
        } else if token.is_error() {
            report_error(&token, &source_chars, "Lexing error");
            panic_mode = true;
            had_error = true;
        } else {
            expression(&mut compiler, &tokens, &mut offset, &mut chunk, &source_chars);
        }
    }
    emit_byte(&mut chunk, OpCode::Return, 99);
    if had_error {
        None
    } else {
        Some(chunk)
    }
}

fn interpret<'a>(vm: &mut VM, source: String) -> InterpretResult<'a> {
    match compile(source) {
        None => vm::InterpretResult::CompileError,
        Some(chunk) => vm.interpret(chunk, true)
    }
}

fn repl() -> Result<()> {
    let mut vm = vm::init_vm();
    loop {
        print!("> ");
        let _ = std::io::stdout().flush();
        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input);
        if input == "\n" {
            println!("");
            break;
        }
        match interpret(&mut vm, input) {
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
    let mut vm = vm::init_vm();
    match interpret(&mut vm, source) {
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
