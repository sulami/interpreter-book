use std::collections::HashMap;

#[derive(Clone,PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Symbol(String),
}

impl Value {
    fn truthy(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Bool(false) => false,
            Value::Int(0) => false,
            Value::Float(f) => *f == 0.0,
            Value::String(s) => s.is_empty(),
            _ => true,
        }
    }

    fn negate(&self) -> Option<Value> {
        match self {
            Value::Int(x) => Some(Value::Int(-x)),
            Value::Float(x) => Some(Value::Float(-x)),
            _ => None,
        }
    }

    fn add(&self, other: Value) -> Option<Value> {
        match (self, other) {
            // float & float -> float
            (Value::Float(a), Value::Float(b)) => Some(Value::Float(a+b)),
            // float & int -> float
            (Value::Int(a), Value::Float(b)) => Some(Value::Float(*a as f64 + b)),
            (Value::Float(a), Value::Int(b)) => Some(Value::Float(a + b as f64)),
            // int & int -> int
            (Value::Int(a), Value::Int(b)) => Some(Value::Int(a+b)),
            _ => None,
        }
    }

    fn subtract(&self, other: Value) -> Option<Value> {
        match (self, other) {
            // float & float -> float
            (Value::Float(a), Value::Float(b)) => Some(Value::Float(a-b)),
            // float & int -> float
            (Value::Int(a), Value::Float(b)) => Some(Value::Float(*a as f64 - b)),
            (Value::Float(a), Value::Int(b)) => Some(Value::Float(a - b as f64)),
            // int & int -> int
            (Value::Int(a), Value::Int(b)) => Some(Value::Int(a-b)),
            _ => None,
        }
    }

    fn multiply(&self, other: Value) -> Option<Value> {
        match (self, other) {
            // float & float -> float
            (Value::Float(a), Value::Float(b)) => Some(Value::Float(a*b)),
            // float & int -> float
            (Value::Int(a), Value::Float(b)) => Some(Value::Float(*a as f64 * b)),
            (Value::Float(a), Value::Int(b)) => Some(Value::Float(a * b as f64)),
            // int & int -> int
            (Value::Int(a), Value::Int(b)) => Some(Value::Int(a*b)),
            _ => None,
        }
    }

    fn divide(&self, other: Value) -> Option<Value> {
        match (self, other) {
            // float & float -> float
            (Value::Float(a), Value::Float(b)) => Some(Value::Float(a/b)),
            // float & int -> float
            (Value::Int(a), Value::Float(b)) => Some(Value::Float(*a as f64 / b)),
            (Value::Float(a), Value::Int(b)) => Some(Value::Float(a / b as f64)),
            // int & int -> also float
            (Value::Int(a), Value::Int(b)) => Some(Value::Float(*a as f64 / b as f64)),
            _ => None,
        }
    }

    fn not(&self) -> Option<Value> {
        match self {
            Value::Bool(b) => Some(Value::Bool(!b)),
            _ => None,
        }
    }

    fn equal(&self, other: Value) -> Option<Value> {
        Some(Value::Bool(*self == other))
    }

    fn greater_than(&self, other: Value) -> Option<Value> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Some(Value::Bool(*a > b)),
            (Value::Int(a), Value::Float(b)) => Some(Value::Bool((*a as f64) > b)),
            (Value::Float(a), Value::Int(b)) => Some(Value::Bool(*a > b as f64)),
            (Value::Float(a), Value::Float(b)) => Some(Value::Bool(*a > b)),
            _ => None,
        }
    }

    fn less_than(&self, other: Value) -> Option<Value> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Some(Value::Bool(*a < b)),
            (Value::Int(a), Value::Float(b)) => Some(Value::Bool((*a as f64) < b)),
            (Value::Float(a), Value::Int(b)) => Some(Value::Bool(*a < b as f64)),
            (Value::Float(a), Value::Float(b)) => Some(Value::Bool(*a < b)),
            _ => None,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Int(x) => write!(f, "{}", x),
            Value::Float(x) => write!(f, "{}", x),
            Value::String(s) => write!(f, "{}", s),
            Value::Symbol(s) => write!(f, "{}", s),
        }
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Value::String(s) => write!(f, "\"{}\"", s),
            _ => write!(f, "{}", self),
        }
    }
}

type ValueArray = Vec<Value>;

#[allow(dead_code)]
#[derive(Debug)]
pub enum OpCode {
    Constant(usize),
    DefineGlobal(usize),
    GetGlobal(usize),
    DefineLocal(usize),
    GetLocal(usize),
    Jump(usize),
    JumpIfFalse(usize),
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    Not,
    Equal,
    GreaterThan,
    LessThan,
    Print,
    Pop,
    Zap(usize),
    Wipe,
    Return,
}

pub type Line = u32;

pub struct Chunk {
    pub code: Vec<OpCode>,
    pub lines: Vec<Line>,
    pub constants: ValueArray,
}

impl Chunk {
    #[allow(dead_code)]
    pub fn disassemble(&self) {
        for i in 0..self.code.len() {
            self.disassemble_instruction(i)
        }
    }

    fn read_constant(&self, index: usize) -> Value {
        match &self.constants[index] {
            Value::Nil => Value::Nil,
            Value::Bool(b) => Value::Bool(*b),
            Value::Int(n) => Value::Int(*n),
            Value::Float(n) => Value::Float(*n),
            Value::String(s) => Value::String(String::from(s)),
            Value::Symbol(s) => Value::Symbol(String::from(s)),
        }
    }

    pub fn write_constant(&mut self, value: Value) -> usize {
        self.constants.append(&mut vec![value]);
        self.constants.len() - 1
    }

    pub fn backpatch_jump(&mut self, idx: usize) {
        let target = self.code.len() - 1;
        match self.code[idx] {
            OpCode::Jump(_) => self.code[idx] = OpCode::Jump(target),
            OpCode::JumpIfFalse(_) => self.code[idx] = OpCode::JumpIfFalse(target),
            _ => panic!("This is not a jump"),
        }
    }

    pub fn write_code(&mut self, op_code: OpCode, line: Line) {
        self.code.append(&mut vec![op_code]);
        self.write_line(line);
    }

    fn write_line(&mut self, line: Line) {
        self.lines.append(&mut vec![line]);
    }

    fn disassemble_instruction(&self, index: usize) {
        let instruction: &OpCode = &self.code[index];
        if index > 0 && &self.lines[index] == &self.lines[index-1] {
            print!("{:04x} {:>5} ", index, "|");
        } else {
            print!("{:04x} {:>5} ", index, &self.lines[index]);
        };
        match instruction {
            OpCode::Constant(ptr) => println!("CONSTANT \t[{:04}] =>\t{:?}", ptr, self.read_constant(*ptr)),
            OpCode::DefineGlobal(ptr) => println!("DEF GLOBAL\t[{:04}] =>\t{:?}", ptr, self.read_constant(*ptr)),
            OpCode::GetGlobal(ptr) => println!("GET GLOBAL\t[{:04}] =>\t{:?}", ptr, self.read_constant(*ptr)),
            OpCode::DefineLocal(ptr) => println!("DEF LOCAL\t[{:04x}]", ptr),
            OpCode::GetLocal(ptr) => println!("GET LOCAL\t[{:04x}]", ptr),
            OpCode::Jump(ptr) => println!("JMP\t\t[{:04x}]", ptr),
            OpCode::JumpIfFalse(ptr) => println!("JMP IF F\t[{:04x}]", ptr),
            OpCode::Negate => println!("NEGATE"),
            OpCode::Add => println!("ADD"),
            OpCode::Subtract => println!("SUBTRACT"),
            OpCode::Multiply => println!("MULTIPLY"),
            OpCode::Divide => println!("DIVIDE"),
            OpCode::Not => println!("NOT"),
            OpCode::Equal => println!("EQUAL"),
            OpCode::GreaterThan => println!("GT"),
            OpCode::LessThan => println!("LT"),
            OpCode::Print => println!("PRINT"),
            OpCode::Pop => println!("POP"),
            OpCode::Zap(ptr) => println!("ZAP\t\t[{:04}]", ptr),
            OpCode::Wipe => println!("WIPE"),
            OpCode::Return => println!("RETURN"),
        }
    }
}

impl std::fmt::Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "code: {:?}\nconstants: {:?}\nlines: {:?}",
               self.code, self.constants, self.lines)
    }
}

pub struct VM {
    ip: usize,
    stack: ValueArray,
    globals: HashMap<String, Value>,
}

pub enum InterpretResult<'a> {
    OK,
    CompileError,
    RuntimeError(&'a str),
}

impl VM {
    fn print_state(&self) {
        println!("== vm state ==");
        println!("ip: {:04x}", self.ip);
        println!("stack: {:?}", self.stack);
        println!("globals: {:?}", self.globals);
    }

    pub fn interpret<'a>(&mut self, chunk: Chunk, debug: bool) -> InterpretResult<'a> {
        self.ip = 0;
        self.stack = vec![];
        loop {
            if debug {
                chunk.disassemble_instruction(self.ip);
            }
            if chunk.code.len() - 1 <= self.ip {
                if debug {
                    self.print_state();
                }
                break InterpretResult::OK;
            }
            match &chunk.code[self.ip] {
                OpCode::Constant(ptr) => {
                    self.stack.push(chunk.read_constant(*ptr));
                }
                OpCode::DefineGlobal(ptr) => {
                    match self.stack.pop() {
                        Some(v) => {
                            let name = chunk.read_constant(*ptr);
                            self.globals.insert(name.to_string(), v);
                        },
                        None => break InterpretResult::RuntimeError("Empty stack"),
                    }
                }
                OpCode::GetGlobal(ptr) => {
                    let name = chunk.read_constant(*ptr);
                    match self.globals.get(&name.to_string()) {
                        Some(v) => self.stack.push(v.clone()),
                        None => break InterpretResult::RuntimeError("Symbol not found"),
                    }
                }
                OpCode::DefineLocal(_ptr) => {
                }
                OpCode::GetLocal(idx) => self.stack.push(self.stack[*idx].clone()),
                OpCode::Jump(ptr) => self.ip = *ptr,
                OpCode::JumpIfFalse(ptr) => {
                    match self.stack.last() {
                        Some(v) => {
                            if !v.truthy() {
                                self.ip = *ptr;
                            }
                        }
                        None => break InterpretResult::RuntimeError("Empty stack"),
                    }
                }
                OpCode::Negate => {
                    match self.stack.pop() {
                        Some(v) => match v.negate() {
                            Some(v) => self.stack.push(v),
                            None => break InterpretResult::RuntimeError("Type error"),
                        }
                        None => break InterpretResult::RuntimeError("Empty stack"),
                    }
                }
                OpCode::Add => {
                    match (self.stack.pop(), self.stack.pop()) {
                        (Some(a), Some(b)) => match b.add(a) {
                            Some(v) => self.stack.push(v),
                            None => break InterpretResult::RuntimeError("Type error"),
                        }
                        (_, None) => break InterpretResult::RuntimeError("Empty stack"),
                        (None, _) => break InterpretResult::RuntimeError("Empty stack"),
                    }
                }
                OpCode::Subtract => {
                    match (self.stack.pop(), self.stack.pop()) {
                        (Some(a), Some(b)) => match b.subtract(a) {
                            Some(v) => self.stack.push(v),
                            None => break InterpretResult::RuntimeError("Type error"),
                        }
                        (_, None) => break InterpretResult::RuntimeError("Empty stack"),
                        (None, _) => break InterpretResult::RuntimeError("Empty stack"),
                    }
                }
                OpCode::Multiply => {
                    match (self.stack.pop(), self.stack.pop()) {
                        (Some(a), Some(b)) => match b.multiply(a) {
                            Some(v) => self.stack.push(v),
                            None => break InterpretResult::RuntimeError("Type error"),
                        }
                        (_, None) => break InterpretResult::RuntimeError("Empty stack"),
                        (None, _) => break InterpretResult::RuntimeError("Empty stack"),
                    }
                }
                OpCode::Divide => {
                    match (self.stack.pop(), self.stack.pop()) {
                        (Some(a), Some(b)) => match b.divide(a) {
                            Some(v) => self.stack.push(v),
                            None => break InterpretResult::RuntimeError("Type error"),
                        }
                        (_, None) => break InterpretResult::RuntimeError("Empty stack"),
                        (None, _) => break InterpretResult::RuntimeError("Empty stack"),
                    }
                }
                OpCode::Not => {
                    match self.stack.pop() {
                        Some(b) => match b.not() {
                            Some(v) => self.stack.push(v),
                            None => break InterpretResult::RuntimeError("Type error"),
                        }
                        None => break InterpretResult::RuntimeError("Empty stack"),
                    }
                }
                OpCode::Equal => {
                    match (self.stack.pop(), self.stack.pop()) {
                        (Some(a), Some(b)) => match b.equal(a) {
                            Some(v) => self.stack.push(v),
                            None => break InterpretResult::RuntimeError("Type error"),
                        }
                        (_, None) => break InterpretResult::RuntimeError("Empty stack"),
                        (None, _) => break InterpretResult::RuntimeError("Empty stack"),
                    }
                }
                OpCode::GreaterThan => {
                    match (self.stack.pop(), self.stack.pop()) {
                        (Some(a), Some(b)) => match b.greater_than(a) {
                            Some(v) => self.stack.push(v),
                            None => break InterpretResult::RuntimeError("Type error"),
                        }
                        (_, None) => break InterpretResult::RuntimeError("Empty stack"),
                        (None, _) => break InterpretResult::RuntimeError("Empty stack"),
                    }
                }
                OpCode::LessThan => {
                    match (self.stack.pop(), self.stack.pop()) {
                        (Some(a), Some(b)) => match b.less_than(a) {
                            Some(v) => self.stack.push(v),
                            None => break InterpretResult::RuntimeError("Type error"),
                        }
                        (_, None) => break InterpretResult::RuntimeError("Empty stack"),
                        (None, _) => break InterpretResult::RuntimeError("Empty stack"),
                    }
                }
                OpCode::Print => {
                    match self.stack.pop() {
                        Some(c) => {
                            println!("{}", c); // TODO raw print without newline
                            self.stack.push(Value::Nil);
                        }
                        None => break InterpretResult::RuntimeError("Empty stack"),
                    }
                }
                OpCode::Pop => {
                    if self.stack.pop().is_none() {
                        break InterpretResult::RuntimeError("Empty stack");
                    }
                }
                OpCode::Zap(ptr) => {
                    if self.stack.len() <= *ptr {
                        break InterpretResult::RuntimeError("Zap out of bounds")
                    }
                    self.stack.remove(*ptr);
                }
                OpCode::Wipe => self.stack.clear(),
                OpCode::Return => {
                    match self.stack.pop() {
                        Some(c) => println!("{}", c),
                        None => break InterpretResult::RuntimeError("Empty stack"),
                    }
                }
            };
            self.ip += 1;
        }
    }
}

pub fn init_vm() -> VM {
    VM{
        ip: 0,
        stack: vec![],
        globals: HashMap::new(),
    }
}
