#[derive(Debug)]
enum OpCode {
    Constant(usize),
    Negate,
    Return,
}

enum Value {
    Float(f64),
}

impl Value {
    fn negate(&self) -> Value {
        match self {
            Value::Float(x) => Value::Float(-x),
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
            OpCode::Constant(ptr) => println!("CONSTANT \t{} \t{}", ptr, self.read_constant(*ptr)),
            OpCode::Negate => println!("NEGATE"),
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

enum InterpretResult {
    OK,
    // CompileError,
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
            let instruction = &self.chunk.code[self.ip];
            if debug {
                self.chunk.disassemble_instruction(self.ip);
            }
            match instruction {
                OpCode::Constant(ptr) => {
                    self.stack.push(self.chunk.read_constant(*ptr));
                }
                OpCode::Negate => {
                    match self.stack.pop() {
                        Some(v) => self.stack.push(v.negate()),
                        None => break InterpretResult::RuntimeError,
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
    let chunk: Chunk = Chunk{
        code: vec![OpCode::Constant(0), OpCode::Negate, OpCode::Return],
        lines: vec![123, 123, 123],
        constants: vec![Value::Float(1.2)],
    };
    // chunk.disassemble();
    init_vm(chunk).interpret(true);
}
