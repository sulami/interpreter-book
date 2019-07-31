enum Value {
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
            (Value::Float(a), Value::Float(b)) => Value::Float(b+a)
        }
    }

    fn subtract(&self, other: Value) -> Value {
        match (self, other) {
            (Value::Float(a), Value::Float(b)) => Value::Float(b-a)
        }
    }

    fn multiply(&self, other: Value) -> Value {
        match (self, other) {
            (Value::Float(a), Value::Float(b)) => Value::Float(b*a)
        }
    }

    fn divide(&self, other: Value) -> Value {
        match (self, other) {
            (Value::Float(a), Value::Float(b)) => Value::Float(b/a)
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
enum OpCode {
    Constant(usize),
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    Return,
}

struct Chunk {
    code: Vec<OpCode>,
    lines: Vec<u32>,
    constants: ValueArray,
}

impl Chunk {
    fn read_constant(&self, index: usize) -> Value {
        match self.constants[index] {
            Value::Float(n) => Value::Float(n)
        }
    }

    #[allow(dead_code)]
    fn disassemble(&self) {
        for i in 0..self.code.len() {
            self.disassemble_instruction(i)
        }
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

struct VM {
    chunk: Chunk,
    ip: usize,
    stack: ValueArray,
}

#[allow(dead_code)]
enum InterpretResult {
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

    fn interpret(mut self, debug: bool) -> InterpretResult {
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
                        (Some(a), Some(b)) => self.stack.push(a.add(b)),
                    }
                }
                OpCode::Subtract => {
                    match (self.stack.pop(), self.stack.pop()) {
                        (_, None) => break InterpretResult::RuntimeError,
                        (None, _) => break InterpretResult::RuntimeError,
                        (Some(a), Some(b)) => self.stack.push(a.subtract(b)),
                    }
                }
                OpCode::Multiply => {
                    match (self.stack.pop(), self.stack.pop()) {
                        (_, None) => break InterpretResult::RuntimeError,
                        (None, _) => break InterpretResult::RuntimeError,
                        (Some(a), Some(b)) => self.stack.push(a.multiply(b)),
                    }
                }
                OpCode::Divide => {
                    match (self.stack.pop(), self.stack.pop()) {
                        (_, None) => break InterpretResult::RuntimeError,
                        (None, _) => break InterpretResult::RuntimeError,
                        (Some(a), Some(b)) => self.stack.push(a.divide(b)),
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

fn init_vm(chunk: Chunk) -> VM {
    VM{
        chunk: chunk,
        ip: 0,
        stack: vec![],
    }
}

fn main() {
    // Negate a constant
    let chunk: Chunk = Chunk{
        code: vec![OpCode::Constant(0), OpCode::Negate, OpCode::Return],
        lines: vec![123, 123, 123],
        constants: vec![Value::Float(1.2)],
    };
    init_vm(chunk).interpret(true);

    // Multiply some constants
    let chunk: Chunk = Chunk{
        code: vec![OpCode::Constant(0),
                   OpCode::Constant(1),
                   OpCode::Add,
                   OpCode::Constant(2),
                   OpCode::Divide,
                   OpCode::Negate,
                   OpCode::Return],
        lines: vec![123, 124, 125, 125, 125, 126, 126],
        constants: vec![Value::Float(1.2), Value::Float(3.4), Value::Float(5.6)],
    };
    // chunk.disassemble();
    init_vm(chunk).interpret(true);
}
