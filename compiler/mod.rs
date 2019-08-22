mod scanner;
pub mod vm;

use self::scanner::{Token, TokenType};
use self::vm::{Chunk, OpCode, Value, VM};

pub type SourceCode = Vec<char>;

pub struct LocalVar {
    name: String,
    depth: usize,
}

pub struct Compiler {
    chunk: Chunk,
    locals: Vec<LocalVar>,
    scope_depth: usize,
    sexp_depth: usize,
    is_main: bool,
}

fn advance(tokens: &Vec<Token>, offset: &mut usize) -> Result<(), String> {
    if *offset < tokens.len() - 1 {
       *offset += 1;
        Ok(())
    } else {
        Err(String::from("Unexpected end of file"))
    }
}

fn do_expressions(compiler: &mut Compiler,
                  tokens: &Vec<Token>,
                  offset: &mut usize,
                  source: &SourceCode)
                  -> Result<(), String> {
    if tokens[*offset].token_type != TokenType::CloseParenthesis {
        try!(expression(compiler, tokens, offset, source));
        // Just keep evaluating in the current scope until we run out
        while tokens[*offset].token_type != TokenType::CloseParenthesis {
            // Pop all but the last value off the stack again
            let token = &tokens[*offset];
            compiler.chunk.write_code(OpCode::Pop, token.line);
            try!(expression(compiler, tokens, offset, source));
        }
    }
    Ok(())
}

fn compile_def(compiler: &mut Compiler,
               tokens: &Vec<Token>,
               offset: &mut usize,
               source: &SourceCode)
               -> Result<(), String> {
    let token = &tokens[*offset];
    // `def` needs to read ahead because the first arg is a raw symbol
    try!(advance(tokens, offset));
    let next_token = &tokens[*offset];
    if next_token.token_type != TokenType::Symbol {
        return Err(String::from("Expected symbol for def"));
    }
    let sym = next_token.get_token(source);
    try!(advance(tokens, offset));
    try!(expression(compiler, tokens, offset, source));
    let idx = compiler.chunk.write_constant(Value::Symbol(sym));
    compiler.chunk.write_code(OpCode::DefineGlobal(idx), token.line);
    Ok(())
}

fn compile_let(compiler: &mut Compiler,
               tokens: &Vec<Token>,
               offset: &mut usize,
               source: &SourceCode)
               -> Result<(), String> {
    let token = &tokens[*offset];
    // Setup a new scope
    try!(advance(tokens, offset));
    compiler.scope_depth += 1;
    // Eval & Setup the bindings
    try!(consume_token(tokens, offset, &TokenType::OpenParenthesis));
    while &tokens[*offset].token_type == &TokenType::OpenParenthesis {
        try!(consume_token(tokens, offset, &TokenType::OpenParenthesis));
        // TODO error if not a symbol
        let binding_token = &tokens[*offset];
        let name = binding_token.get_token(source);
        try!(advance(tokens, offset));
        try!(expression(compiler, tokens, offset, source));
        // chunk.write_code(OpCode::DefineLocal(compiler.locals.len()), binding_token.line);
        compiler.locals.append(&mut vec![LocalVar{
            name: name.to_string(),
            depth: compiler.scope_depth,
        }]);
        try!(consume_token(tokens, offset, &TokenType::CloseParenthesis));
    }
    try!(consume_token(tokens, offset, &TokenType::CloseParenthesis));
    // Eval the inner expressions
    try!(do_expressions(compiler, tokens, offset, source));
    // Zap the local scope off the stack when it ends
    compiler.scope_depth -= 1;
    let local_count = compiler.locals.len();
    for i in 0..local_count {
        let idx = local_count - i - 1;
        let l = &compiler.locals[idx];
        if compiler.scope_depth < l.depth {
            compiler.locals.pop();
            compiler.chunk.write_code(OpCode::Zap(idx), token.line);
        } else {
            break
        }
    }
    Ok(())
}

fn compile_when(compiler: &mut Compiler,
                tokens: &Vec<Token>,
                offset: &mut usize,
                source: &SourceCode)
                -> Result<(), String> {
    let token = &tokens[*offset];
    try!(advance(tokens, offset));
    // Eval the condition onto the stack
    try!(expression(compiler, tokens, offset, source));
    // Write a provisional JMP instruction and note the position
    compiler.chunk.write_code(OpCode::JumpIfFalse(0), token.line);
    let jmp_idx = compiler.chunk.code.len() - 1;
    // Pop the conditional value
    compiler.chunk.write_code(OpCode::Pop, token.line);
    // Eval the body
    try!(do_expressions(compiler, tokens, offset, source));
    // Backpatch the end of the body into the JMP instruction
    compiler.chunk.backpatch_jump(jmp_idx);
    Ok(())
}

fn compile_if(compiler: &mut Compiler,
              tokens: &Vec<Token>,
              offset: &mut usize,
              source: &SourceCode)
              -> Result<(), String> {
    let token = &tokens[*offset];
    try!(advance(tokens, offset));
    // Eval the condition onto the stack
    try!(expression(compiler, tokens, offset, source));
    // Write a provisional JMP instruction and note the position
    compiler.chunk.write_code(OpCode::JumpIfFalse(0), token.line);
    let sad_jmp_idx = compiler.chunk.code.len() - 1;
    // Pop the conditional value on the happy path
    compiler.chunk.write_code(OpCode::Pop, token.line);
    // Eval the happy path body
    try!(expression(compiler, tokens, offset, source));
    // Write a provisional JMP instruction to pass the sad path
    compiler.chunk.write_code(OpCode::Jump(0), token.line);
    let happy_jmp_idx = compiler.chunk.code.len() - 1;
    // Pop the conditional value on the sad path
    compiler.chunk.write_code(OpCode::Pop, token.line);
    // Backpatch the end of the happy path body into the first JMP instruction
    compiler.chunk.backpatch_jump(sad_jmp_idx);
    // Eval the sad path body
    try!(expression(compiler, tokens, offset, source));
    // Backpatch the end of the sad path body into the second JMP instruction
    compiler.chunk.backpatch_jump(happy_jmp_idx);
    Ok(())
}

fn compile_and(compiler: &mut Compiler,
               tokens: &Vec<Token>,
               offset: &mut usize,
               source: &SourceCode)
               -> Result<(), String> {
    let token = &tokens[*offset];
    try!(advance(tokens, offset));
    // TODO implement n-arity
    // Eval the first argument
    try!(expression(compiler, tokens, offset, source));
    // Write a provisional JMP instruction and note the position
    compiler.chunk.write_code(OpCode::JumpIfFalse(0), token.line);
    let jmp_idx = compiler.chunk.code.len() - 1;
    compiler.chunk.write_code(OpCode::Pop, token.line);
    // Eval the second argument
    try!(expression(compiler, tokens, offset, source));
    // Backpatch the JMP instruction to skip eval of the second argument
    // if the first one is falsy
    compiler.chunk.backpatch_jump(jmp_idx);
    Ok(())
}

fn compile_or(compiler: &mut Compiler,
              tokens: &Vec<Token>,
              offset: &mut usize,
              source: &SourceCode)
              -> Result<(), String> {
    let token = &tokens[*offset];
    try!(advance(tokens, offset));
    // TODO implement n-arity
    // Eval the first argument
    try!(expression(compiler, tokens, offset, source));
    // Jump past the next jump if the first arg is falsy
    compiler.chunk.write_code(OpCode::JumpIfFalse(0), token.line);
    let happy_jmp_idx = compiler.chunk.code.len() - 1;
    // Jump past the second arg otherwise
    compiler.chunk.write_code(OpCode::Jump(0), token.line);
    let sad_jmp_idx = compiler.chunk.code.len() - 1;
    // The first JMP goes here
    compiler.chunk.backpatch_jump(happy_jmp_idx);
    compiler.chunk.write_code(OpCode::Pop, token.line);
    // Eval the second argument
    try!(expression(compiler, tokens, offset, source));
    // The second JMP goes here
    compiler.chunk.backpatch_jump(sad_jmp_idx);
    Ok(())
}

fn compile_while(compiler: &mut Compiler,
                 tokens: &Vec<Token>,
                 offset: &mut usize,
                 source: &SourceCode)
                 -> Result<(), String> {
    let token = &tokens[*offset];
    try!(advance(tokens, offset));
    // Set the loop starting point
    let loop_start_idx = compiler.chunk.code.len() - 1;
    // Eval the condition
    try!(expression(compiler, tokens, offset, source));
    // This JMP termiates the loop
    compiler.chunk.write_code(OpCode::JumpIfFalse(0), token.line);
    let loop_end_jmp_idx = compiler.chunk.code.len() - 1;
    compiler.chunk.write_code(OpCode::Pop, token.line);
    // Eval the body
    try!(do_expressions(compiler, tokens, offset, source));
    // Discard the last value
    compiler.chunk.write_code(OpCode::Pop, token.line);
    // Jump back to the condition
    compiler.chunk.write_code(OpCode::Jump(loop_start_idx), token.line);
    // Jump to here if we're done looping
    compiler.chunk.backpatch_jump(loop_end_jmp_idx);
    compiler.chunk.write_code(OpCode::Pop, token.line);
    Ok(())
}

fn compile_defn(compiler: &mut Compiler,
                tokens: &Vec<Token>,
                offset: &mut usize,
                source: &SourceCode)
                -> Result<(), String> {
    let start_token = &tokens[*offset];
    // Name
    try!(advance(tokens, offset));
    let name_token = &tokens[*offset];
    if name_token.token_type != TokenType::Symbol {
        return Err(format!("Function name needs to be a symbol, got {}", name_token.token_type))
    }
    let fn_name = name_token.get_token(source);
    // Parameters
    let mut argc = 0;
    let inner_chunk = Chunk{
        code: vec![],
        constants: vec![],
        lines: vec![],
    };
    let mut inner_compiler = Compiler{
        chunk: inner_chunk,
        locals: vec![],
        scope_depth: 0,
        sexp_depth: 0,
        is_main: false,
    };
    try!(advance(tokens, offset));
    try!(consume_token(tokens, offset, &TokenType::OpenParenthesis));
    while &tokens[*offset].token_type != &TokenType::CloseParenthesis {
        argc += 1;
        let binding_token = &tokens[*offset];
        if binding_token.token_type != TokenType::Symbol {
            return Err(format!("Function binding must be a symbol, got {}", binding_token.token_type));
        }
        inner_compiler.locals.append(&mut vec![LocalVar{
            name: binding_token.get_token(source).to_string(),
            depth: inner_compiler.scope_depth,
        }]);
        try!(advance(tokens, offset));
    }
    try!(consume_token(tokens, offset, &TokenType::CloseParenthesis));
    // Body
    // TODO reuse this code between this and compile()
    while &tokens[*offset].token_type != &TokenType::CloseParenthesis {
        let token = &tokens[*offset];
        if token.is_error() {
            return Err(format!("Lexing error: {}", token.token_type));
        } else {
            let exp = expression(&mut inner_compiler, &tokens, offset, &source);
            if exp.is_err() {
                return Err(exp.err().unwrap());
            }
        }
    }
    inner_compiler.chunk.write_code(OpCode::Return, 99);
    // Write function
    let idx = compiler.chunk.write_constant(Value::Function(fn_name, argc, inner_compiler.chunk));
    compiler.chunk.write_code(OpCode::Constant(idx), start_token.line);
    compiler.chunk.write_code(OpCode::DefineGlobal(idx), start_token.line);
    Ok(())
}

fn compile_fn_call(compiler: &mut Compiler,
                   tokens: &Vec<Token>,
                   offset: &mut usize,
                   source: &SourceCode)
                   -> Result<(), String> {
    let token = &tokens[*offset];
    let fn_name = token.get_token(source);
    let mut custom = false;
    let mut ops = match fn_name.as_str() {
        "+" => vec![OpCode::Add],
        "-" => vec![OpCode::Subtract],
        "*" => vec![OpCode::Multiply],
        "/" => vec![OpCode::Divide],
        "not" => vec![OpCode::Not],
        "=" => vec![OpCode::Equal],
        ">" => vec![OpCode::GreaterThan],
        ">=" => vec![OpCode::LessThan, OpCode::Not],
        "<" => vec![OpCode::LessThan],
        "<=" => vec![OpCode::GreaterThan, OpCode::Not],
        "print" => vec![OpCode::Print],
        _ => {
            custom = true;
            // Gets filled in later
            vec![]
        }
    };
    if custom {
        // Custom functions get pushed to the stack first.
        try!(expression(compiler, tokens, offset, source));
    } else {
        try!(advance(tokens, offset));
    }
    let mut argc = 0;
    while tokens[*offset].token_type != TokenType::CloseParenthesis {
        argc += 1;
        try!(expression(compiler, tokens, offset, source));
    }
    if custom {
        ops = vec![OpCode::Call(argc)];
    }
    for op in ops {
        compiler.chunk.write_code(op, token.line);
    }
    Ok(())
}

fn compile_sexp(compiler: &mut Compiler,
                tokens: &Vec<Token>,
                offset: &mut usize,
                source: &SourceCode)
                -> Result<(), String> {
    compiler.sexp_depth += 1;
    try!(advance(tokens, offset));
    let token = &tokens[*offset];
    if token.token_type != TokenType::Symbol {
        return Err(format!("Function name must be a symbol, got {}", token.token_type));
    }
    let fn_name = token.get_token(source);
    match fn_name.as_str() {
        "def" => try!(compile_def(compiler, tokens, offset, source)),
        "let" => try!(compile_let(compiler, tokens, offset, source)),
        "when" => try!(compile_when(compiler, tokens, offset, source)),
        "if" => try!(compile_if(compiler, tokens, offset, source)),
        "and" => try!(compile_and(compiler, tokens, offset, source)),
        "or" => try!(compile_or(compiler, tokens, offset, source)),
        "while" => try!(compile_while(compiler, tokens, offset, source)),
        "defn" => try!(compile_defn(compiler, tokens, offset, source)),
        "do" => {
            try!(advance(tokens, offset));
            try!(do_expressions(compiler, tokens, offset, source));
        }
        _ => try!(compile_fn_call(compiler, tokens, offset, source)),
    }
    try!(consume_token(tokens, offset, &TokenType::CloseParenthesis));
    compiler.sexp_depth -= 1;
    Ok(())
}

fn expression(compiler: &mut Compiler,
              tokens: &Vec<Token>,
              offset: &mut usize,
              source: &SourceCode)
              -> Result<(), String> {
    let token = &tokens[*offset];
    match token.token_type {
        TokenType::OpenParenthesis => try!(compile_sexp(compiler, tokens, offset, source)),
        TokenType::Nil => {
            let idx = compiler.chunk.write_constant(Value::Nil);
            compiler.chunk.write_code(OpCode::Constant(idx), token.line);
            try!(advance(tokens, offset));
        }
        TokenType::Bool => {
            let val: bool = token.get_token(source) == "true";
            let idx = compiler.chunk.write_constant(Value::Bool(val));
            compiler.chunk.write_code(OpCode::Constant(idx), token.line);
            try!(advance(tokens, offset));
        }
        TokenType::Int => {
            let val: i64 = token.get_token(source).parse().unwrap();
            let idx = compiler.chunk.write_constant(Value::Int(val));
            compiler.chunk.write_code(OpCode::Constant(idx), token.line);
            try!(advance(tokens, offset));
        }
        TokenType::Float => {
            let mut raw_val = token.get_token(source);
            if raw_val.starts_with(".") {
                // Parse ".3" as 0.3
                raw_val.insert_str(0, "0");
            }
            let val: f64 = raw_val.parse().unwrap();
            let idx = compiler.chunk.write_constant(Value::Float(val));
            compiler.chunk.write_code(OpCode::Constant(idx), token.line);
            try!(advance(tokens, offset));
        }
        TokenType::Keyword => {
            println!("parsed a keyword: {}", token.get_token(source));
            try!(advance(tokens, offset));
        }
        TokenType::String => {
            let val = token.get_token(source);
            let idx = compiler.chunk.write_constant(Value::String(val));
            compiler.chunk.write_code(OpCode::Constant(idx), token.line);
            try!(advance(tokens, offset));
        }
        TokenType::Symbol => {
            let val = token.get_token(source);
            let local_count = compiler.locals.len();
            let mut is_local = false;
            for i in 0..local_count {
                let idx = local_count - i - 1;
                if compiler.locals[idx].name == val {
                    compiler.chunk.write_code(OpCode::GetLocal(idx), token.line);
                    is_local = true;
                    break
                }
            }
            if !is_local {
                let idx = compiler.chunk.write_constant(Value::Symbol(val));
                compiler.chunk.write_code(OpCode::GetGlobal(idx), token.line);
            }
            try!(advance(tokens, offset));
        }
        TokenType::EOF => {
            try!(advance(tokens, offset));
        }
        _ => panic!("Token type not implemented: {}", token.token_type),
    };
    if compiler.is_main && compiler.sexp_depth == 0 {
        compiler.chunk.write_code(OpCode::Pop, token.line);
    }
    Ok(())
}

fn consume_token(tokens: &Vec<Token>, offset: &mut usize, expected_type: &TokenType)
                 -> Result<(), String> {
    let token = &tokens[*offset];
    if token.token_type == *expected_type {
        try!(advance(tokens, offset));
        Ok(())
    } else {
        Err(format!("Expected {}, got {}", *expected_type, token.token_type))
    }
}

fn compile(source: &SourceCode, debug: bool) -> Result<Chunk, String> {
    let chunk = Chunk{
        code: vec![],
        constants: vec![],
        lines: vec![],
    };
    let mut compiler = Compiler{
        chunk: chunk,
        locals: vec![],
        scope_depth: 0,
        sexp_depth: 0,
        is_main: true,
    };
    let tokens = scanner::scan(&source, debug);
    let mut offset = 0;
    let token_count = tokens.len();
    while offset < token_count - 1 {
        let token = &tokens[offset];
        if token.is_error() {
            return Err(format!("Lexing error: {}", token.token_type));
        } else {
            let exp = expression(&mut compiler, &tokens, &mut offset, &source);
            if exp.is_err() {
                return Err(exp.err().unwrap());
            }
        }
    }
    compiler.chunk.write_code(OpCode::Return, 99);
    Ok(compiler.chunk)
}

pub fn interpret<'a>(vm: &mut VM, source: String, debug: bool) -> Result<(), String> {
    let source_chars: SourceCode = source.chars().collect();
    let chunk = try!(compile(&source_chars, debug));
    vm.interpret(chunk, debug)
}
