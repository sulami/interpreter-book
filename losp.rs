enum OpCode {
    Return,
}

type Chunk = [OpCode];

fn disassemble_instruction(offset: usize, instruction: &OpCode) {
    print!("{} ", offset);
    match instruction {
        OpCode::Return => println!("RETURN"),
    }
}

fn disassemble_chunk(chunk: &Chunk, name: &str) {
    println!("== {} ==", name);
    for (i, instruction) in chunk.iter().enumerate() {
        disassemble_instruction(i, instruction)
    }
}

fn main() {
    let chunk = [OpCode::Return, OpCode::Return];
    disassemble_chunk(&chunk, "test chunk");
}
