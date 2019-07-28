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

    fn disassemble(&self, name: &str) {
        println!("== {} ==", name);
        for i in 0..self.code.len() {
            disassemble_instruction(self, i)
        }
    }
}

fn disassemble_instruction(chunk: &Chunk, offset: usize) {
    let instruction: &OpCode = &chunk.code[offset];
    if offset > 0 && &chunk.lines[offset] == &chunk.lines[offset-1] {
        print!("{:04x} {:>5} ", offset, "|");
    } else {
        print!("{:04x} {:>5} ", offset, &chunk.lines[offset]);
    };
    match instruction {
        OpCode::Constant(ptr) => println!("CONSTANT \t{} \t{}", ptr, chunk.read_constant(*ptr)),
        OpCode::Return => println!("RETURN"),
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
    chunk.disassemble("test chunk");
}
