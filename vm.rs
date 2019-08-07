pub enum Value {
    Nil,
    Bool(bool),
    Float(f64),
}

impl Value {
    fn negate(&self) -> Option<Value> {
        match self {
            Value::Float(x) => Some(Value::Float(-x)),
            _ => None,
        }
    }

    fn add(&self, other: Value) -> Option<Value> {
        match (self, other) {
            (Value::Float(a), Value::Float(b)) => Some(Value::Float(a+b)),
            _ => None,
        }
    }

    fn subtract(&self, other: Value) -> Option<Value> {
        match (self, other) {
            (Value::Float(a), Value::Float(b)) => Some(Value::Float(a-b)),
            _ => None,
        }
    }

    fn multiply(&self, other: Value) -> Option<Value> {
        match (self, other) {
            (Value::Float(a), Value::Float(b)) => Some(Value::Float(a*b)),
            _ => None,
        }
    }

    fn divide(&self, other: Value) -> Option<Value> {
        match (self, other) {
            (Value::Float(a), Value::Float(b)) => Some(Value::Float(a/b)),
            _ => None,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Float(x) => write!(f, "{}", x),
        }
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let t = match self {
            Value::Float(_) => "float: ",
            _ => "",
        };
        write!(f, "{}{}", t, self)
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
        match self.constants[index] {
            Value::Nil => Value::Nil,
            Value::Bool(b) => Value::Bool(b),
            Value::Float(n) => Value::Float(n),
        }
    }

    pub fn write_constant(&mut self, value: Value) -> usize {
        self.constants.append(&mut vec![value]);
        self.constants.len() - 1
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

pub enum InterpretResult<'a> {
    OK,
    CompileError,
    RuntimeError(&'a str),
}

impl VM {
    #[allow(dead_code)]
    fn print_state(self) {
        println!("== vm state ==");
        println!("ip: {}", self.ip);
        println!("stack: {:?}", self.stack);
        println!("{:?}", self.chunk);
    }

    pub fn interpret<'a>(mut self, debug: bool) -> InterpretResult<'a> {
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
                        Some(v) => match v.negate() {
                            Some(v) => self.stack.push(v),
                            None => break InterpretResult::RuntimeError(
                                "Cannot negate"
                            ),
                        }
                        None => break InterpretResult::RuntimeError(
                            "Tried to pop empty stack"),
                    }
                }
                OpCode::Add => {
                    match (self.stack.pop(), self.stack.pop()) {
                        (Some(a), Some(b)) => match b.add(a) {
                            Some(v) => self.stack.push(v),
                            None => break InterpretResult::RuntimeError(""),
                        }
                        (_, None) => break InterpretResult::RuntimeError(""),
                        (None, _) => break InterpretResult::RuntimeError(""),
                    }
                }
                OpCode::Subtract => {
                    match (self.stack.pop(), self.stack.pop()) {
                        (Some(a), Some(b)) => match b.subtract(a) {
                            Some(v) => self.stack.push(v),
                            None => break InterpretResult::RuntimeError(""),
                        }
                        (_, None) => break InterpretResult::RuntimeError(""),
                        (None, _) => break InterpretResult::RuntimeError(""),
                    }
                }
                OpCode::Multiply => {
                    match (self.stack.pop(), self.stack.pop()) {
                        (Some(a), Some(b)) => match b.multiply(a) {
                            Some(v) => self.stack.push(v),
                            None => break InterpretResult::RuntimeError(""),
                        }
                        (_, None) => break InterpretResult::RuntimeError(""),
                        (None, _) => break InterpretResult::RuntimeError(""),
                    }
                }
                OpCode::Divide => {
                    match (self.stack.pop(), self.stack.pop()) {
                        (Some(a), Some(b)) => match b.divide(a) {
                            Some(v) => self.stack.push(v),
                            None => break InterpretResult::RuntimeError(""),
                        }
                        (_, None) => break InterpretResult::RuntimeError(""),
                        (None, _) => break InterpretResult::RuntimeError(""),
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
