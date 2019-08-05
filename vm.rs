pub enum Value {
    Float(f64),
}

impl Value {
    fn negate(&self) -> Value {
        match self {
            Value::Float(x) => Value::Float(-x),
        }
    }

    fn add(&self, other: Value) -> Value {
        match (self, other) {
            (Value::Float(a), Value::Float(b)) => Value::Float(a+b)
        }
    }

    fn subtract(&self, other: Value) -> Value {
        match (self, other) {
            (Value::Float(a), Value::Float(b)) => Value::Float(a-b)
        }
    }

    fn multiply(&self, other: Value) -> Value {
        match (self, other) {
            (Value::Float(a), Value::Float(b)) => Value::Float(a*b)
        }
    }

    fn divide(&self, other: Value) -> Value {
        match (self, other) {
            (Value::Float(a), Value::Float(b)) => Value::Float(a/b)
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Value::Float(x) => write!(f, "{}", x),
        }
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Value::Float(x) => write!(f, "{}", x),
        }
    }
}

type ValueArray = Vec<Value>;

#[allow(dead_code)]
#[derive(Debug)]
pub enum OpCode {
    Constant(usize),
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    Return,
}

pub struct Chunk {
    pub code: Vec<OpCode>,
    pub lines: Vec<u32>,
    pub constants: ValueArray,
}

impl Chunk {
    fn read_constant(&self, index: usize) -> Value {
        match self.constants[index] {
            Value::Float(n) => Value::Float(n)
        }
    }

    #[allow(dead_code)]
    pub fn disassemble(&self) {
        for i in 0..self.code.len() {
            self.disassemble_instruction(i)
        }
    }

    pub fn write_constant(&mut self, value: Value) {
        self.constants.append(&mut vec![value]);
    }

    pub fn write_line(&mut self, line: u32) {
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
            OpCode::Constant(ptr) => println!("CONSTANT \t[{}] =>\t{}", ptr, self.read_constant(*ptr)),
            OpCode::Negate => println!("NEGATE"),
            OpCode::Add => println!("ADD"),
            OpCode::Subtract => println!("SUBTRACT"),
            OpCode::Multiply => println!("MULTIPLY"),
            OpCode::Divide => println!("DIVIDE"),
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
    chunk: Chunk,
    ip: usize,
    stack: ValueArray,
}

pub enum InterpretResult {
    OK,
    CompileError,
    RuntimeError,
}

impl VM {
    #[allow(dead_code)]
    fn print_state(self) {
        println!("== vm state ==");
        println!("ip: {}", self.ip);
        println!("stack: {:?}", self.stack);
        println!("{:?}", self.chunk);
    }

    pub fn interpret(mut self, debug: bool) -> InterpretResult {
        loop {
            if debug {
                self.chunk.disassemble_instruction(self.ip);
            }
            match &self.chunk.code[self.ip] {
                OpCode::Constant(ptr) => {
                    self.stack.push(self.chunk.read_constant(*ptr));
                }
                OpCode::Negate => {
                    match self.stack.pop() {
                        Some(v) => self.stack.push(v.negate()),
                        None => break InterpretResult::RuntimeError,
                    }
                }
                OpCode::Add => {
                    match (self.stack.pop(), self.stack.pop()) {
                        (_, None) => break InterpretResult::RuntimeError,
                        (None, _) => break InterpretResult::RuntimeError,
                        (Some(a), Some(b)) => self.stack.push(b.add(a)),
                    }
                }
                OpCode::Subtract => {
                    match (self.stack.pop(), self.stack.pop()) {
                        (_, None) => break InterpretResult::RuntimeError,
                        (None, _) => break InterpretResult::RuntimeError,
                        (Some(a), Some(b)) => self.stack.push(b.subtract(a)),
                    }
                }
                OpCode::Multiply => {
                    match (self.stack.pop(), self.stack.pop()) {
                        (_, None) => break InterpretResult::RuntimeError,
                        (None, _) => break InterpretResult::RuntimeError,
                        (Some(a), Some(b)) => self.stack.push(b.multiply(a)),
                    }
                }
                OpCode::Divide => {
                    match (self.stack.pop(), self.stack.pop()) {
                        (_, None) => break InterpretResult::RuntimeError,
                        (None, _) => break InterpretResult::RuntimeError,
                        (Some(a), Some(b)) => self.stack.push(b.divide(a)),
                    }
                }
                OpCode::Return => {
                    match self.stack.pop() {
                        Some(c) => println!("{}", c),
                        None => (),
                    }
                    if debug {
                        self.print_state();
                    }
                    break InterpretResult::OK
                }
            };
            self.ip += 1;
        }
    }
}

pub fn init_vm(chunk: Chunk) -> VM {
    VM{
        chunk: chunk,
        ip: 0,
        stack: vec![],
    }
}
