use std::fmt as fmt;

/// A single opcode
enum OpCode {
    /// Returns a single constant from the constant pool
    Constant(usize),
    /// Noop
    Return,
}

/// A constant value
enum Value {
    Float(f64)
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Value::Float(x) => write!(f, "{}", x),
        }
    }
}

/// Constant pool
type ValueArray = Vec<Value>;

/// Variable size buffer of opcodes
struct Chunk {
    code: Vec<OpCode>,
    constants: ValueArray,
}

/// Prints out a disassembled instruction
fn disassemble_instruction(chunk: &Chunk, offset: usize) {
    print!("{:x} ", offset);
    let instruction: &OpCode = &chunk.code[offset];
    match instruction {
        OpCode::Constant(ptr) => println!("CONSTANT \t{} \t{}", ptr, chunk.constants[*ptr]),
        OpCode::Return => println!("RETURN"),
    }
}

/// Prints out a disassembled chunk
fn disassemble_chunk(chunk: &Chunk, name: &str) {
    println!("== {} ==", name);
    for i in 0..chunk.code.len() {
        disassemble_instruction(chunk, i)
    }
}

fn main() {
    let constant_pool: ValueArray = vec![Value::Float(1.2)];
    let chunk: Chunk = Chunk{
        code: vec![OpCode::Constant(0), OpCode::Return],
        constants: constant_pool,
    };
    disassemble_chunk(&chunk, "test chunk");
}
