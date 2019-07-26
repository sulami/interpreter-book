use std::fmt as fmt;

/// A single opcode
enum OpCode {
    /// Returns a single constant from the constant pool
    Constant(usize),
    /// Noop
    Return,
}

/// Variable size buffer of opcodes
type Chunk = [OpCode];

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
type ValueArray = [Value];

/// Prints out a disassembled instruction
fn disassemble_instruction(offset: usize, instruction: &OpCode, constant_pool: &ValueArray) {
    print!("{:x} ", offset);
    match instruction {
        OpCode::Constant(ptr) => println!("CONSTANT \t{} \t{}", ptr, constant_pool[*ptr]),
        OpCode::Return => println!("RETURN"),
    }
}

/// Prints out a disassembled chunk
fn disassemble_chunk(chunk: &Chunk, name: &str, constant_pool: &ValueArray) {
    println!("== {} ==", name);
    for (i, instruction) in chunk.iter().enumerate() {
        disassemble_instruction(i, instruction, constant_pool)
    }
}

fn main() {
    let constant_pool: &ValueArray = &[Value::Float(1.2)];
    let chunk: &Chunk = &[OpCode::Constant(0), OpCode::Return];
    disassemble_chunk(chunk, "test chunk", constant_pool);
}
