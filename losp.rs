enum OpCode {
    /// Returns a single constant from the constant pool
    Constant(usize),
    /// Noop
    Return,
}

enum Value {
    Float(f64),
}

impl std::fmt::Display for Value {
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
    fn read_constant(&self, index: usize) -> &Value {
        &self.constants[index]
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
            OpCode::Return => println!("RETURN"),
        }
    }
}

struct VM<'a> {
    chunk: &'a Chunk,
    ip: usize,
}

enum InterpretResult {
    OK,
    // CompileError,
    // RuntimeError,
}

impl<'a> VM<'a> {
    fn interpret(mut self, debug: bool) -> InterpretResult {
        loop {
            let instruction = &self.chunk.code[self.ip];
            if debug {
                self.chunk.disassemble_instruction(self.ip);
            }
            match instruction {
                OpCode:: Constant(ptr) => println!("{}", self.chunk.read_constant(*ptr)),
                OpCode::Return => break InterpretResult::OK,
            }
            self.ip += 1;
        }
    }
}

fn init_vm(chunk: &Chunk) -> VM {
    VM{
        chunk: chunk,
        ip: 0,
    }
}

fn main() {
    let lines = vec![123, 123];
    let constant_pool: ValueArray = vec![Value::Float(1.2)];
    let chunk: Chunk = Chunk{
        code: vec![OpCode::Constant(0), OpCode::Return],
        lines: lines,
        constants: constant_pool,
    };
    chunk.disassemble();
    init_vm(&chunk).interpret(false);
}
