use std::collections::HashMap;

#[derive(Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Symbol(String),
    Function(String, Chunk),
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

    fn negate(&self) -> Result<Value, String> {
        match self {
            Value::Int(x) => Ok(Value::Int(-x)),
            Value::Float(x) => Ok(Value::Float(-x)),
            _ => Err(format!("Cannot negate {}", self)),
        }
    }

    fn add(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            // float & float -> float
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(*a + *b)),
            // float & int -> float
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f64)),
            // int & int -> int
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(*a + *b)),
            _ => Err(format!("Cannot add {} to {}", other, self)),
        }
    }

    fn subtract(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            // float & float -> float
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(*a - *b)),
            // float & int -> float
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(*a - *b as f64)),
            // int & int -> int
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(*a - *b)),
            _ => Err(format!("Cannot subtract {} from {}", other, self)),
        }
    }

    fn multiply(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            // float & float -> float
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(*a * *b)),
            // float & int -> float
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(*a * *b as f64)),
            // int & int -> int
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(*a * *b)),
            _ => Err(format!("Cannot multiply {} with {}", other, self)),
        }
    }

    fn divide(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            // float & float -> float
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(*a / *b)),
            // float & int -> float
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 / *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(*a / *b as f64)),
            // int & int -> also float
            (Value::Int(a), Value::Int(b)) => Ok(Value::Float(*a as f64 / *b as f64)),
            _ => Err(format!("Cannot divide {} by {}", other, self)),
        }
    }

    fn not(&self) -> Value {
        Value::Bool(!self.truthy())
    }

    fn equal(&self, other: &Value) -> Value {
        let b = match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Bool(x), Value::Bool(y)) => x == y,
            (Value::Int(x), Value::Int(y)) => x == y,
            (Value::Float(x), Value::Float(y)) => x == y,
            (Value::String(x), Value::String(y)) => x == y,
            (Value::Symbol(x), Value::Symbol(y)) => x == y,
            (Value::Function(x, _), Value::Function(y, _)) => x == y,
            _ => false,
        };
        Value::Bool(b)
    }

    fn greater_than(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(*a > *b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) > *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(*a > *b as f64)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(*a > *b)),
            _ => Err(format!("Cannot compare {} with {}", other, self)),
        }
    }

    fn less_than(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(*a < *b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) < *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(*a < *b as f64)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(*a < *b)),
            _ => Err(format!("Cannot compare {} with {}", other, self)),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Int(x) => write!(f, "{}", x),
            Value::Float(x) => write!(f, "{:?}", x),
            Value::String(s) => write!(f, "{}", s),
            Value::Symbol(s) => write!(f, "{}", s),
            Value::Function(s, _) => write!(f, "{}", s),
        }
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Function(s, _) => write!(f, "fn<{}>", s),
            _ => write!(f, "{}", self),
        }
    }
}

type ValueArray = Vec<Value>;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum OpCode {
    Constant(usize),
    DefineGlobal(usize),
    GetGlobal(usize),
    DefineLocal(usize),
    GetLocal(usize),
    Jump(usize),
    JumpIfFalse(usize),
    Call(usize),
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

#[derive(Clone)]
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
            Value::Function(s, c) => Value::Function(String::from(s), c.clone()),
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
            OpCode::Constant(ptr) => println!("CONSTANT\t[{:04}] =>\t{:?}", ptr, self.read_constant(*ptr)),
            OpCode::DefineGlobal(ptr) => println!("DEF GLOBAL\t[{:04}] =>\t{:?}", ptr, self.read_constant(*ptr)),
            OpCode::GetGlobal(ptr) => println!("GET GLOBAL\t[{:04}] =>\t{:?}", ptr, self.read_constant(*ptr)),
            OpCode::DefineLocal(ptr) => println!("DEF LOCAL\t[{:04x}]", ptr),
            OpCode::GetLocal(ptr) => println!("GET LOCAL\t[{:04x}]", ptr),
            OpCode::Jump(ptr) => println!("JMP\t\t[{:04x}]", ptr),
            OpCode::JumpIfFalse(ptr) => println!("JMP IF F\t[{:04x}]", ptr),
            OpCode::Call(argc) => println!("CALL\t\t[{:4}]", argc),
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

pub struct CallFrame {
    fn_name: String,
    ip: usize,
    stack_start: usize,
}

pub struct VM {
    stack: ValueArray,
    globals: HashMap<String, Value>,
    call_stack: Vec<CallFrame>,
}

fn runtime_error(msg: &str) -> Result<(), String> {
    Err(String::from(msg))
}

impl VM {
    fn print_state(&self) {
        println!("== vm state ==");
        println!("ip: {:04x}", self.current_frame().ip);
        println!("stack: {:?}", self.stack);
        println!("globals: {:?}", self.globals);
    }

    fn current_frame(&self) -> &CallFrame {
        self.call_stack.last().unwrap()
    }

    fn current_frame_mut(&mut self) -> &mut CallFrame {
        self.call_stack.last_mut().unwrap()
    }

    fn pop(&mut self) -> Result<Value, String> {
        if self.stack.is_empty() {
            Err(String::from("Empty stack"))
        } else {
            Ok(self.stack.pop().unwrap())
        }
    }

    fn peek(&mut self) -> Result<&Value, String> {
        if self.stack.is_empty() {
            Err(String::from("Empty stack"))
        } else {
            Ok(self.stack.last().unwrap())
        }
    }

    fn pick(&mut self, offset: usize) -> Result<&Value, String> {
        if self.stack.len() <= offset {
            Err(String::from("Pick out of bounds"))
        } else {
            Ok(&self.stack[self.stack.len() - offset - 1])
        }
    }

    pub fn interpret<'a>(&mut self, chunk: Chunk, debug: bool) -> Result<(), String> {
        self.stack = vec![];
        loop {
            if debug {
                chunk.disassemble_instruction(self.current_frame().ip);
            }
            if chunk.code.len() - 1 <= self.current_frame().ip {
                if debug {
                    self.print_state();
                }
                break Ok(())
            }
            match &chunk.code[self.current_frame().ip] {
                OpCode::Constant(ptr) => {
                    self.stack.push(chunk.read_constant(*ptr));
                }
                OpCode::DefineGlobal(ptr) => {
                    let v = try!(self.pop());
                    let name = chunk.read_constant(*ptr);
                    self.globals.insert(name.to_string(), v);
                    self.stack.push(Value::Symbol(name.to_string()));
                }
                OpCode::GetGlobal(ptr) => {
                    let name = chunk.read_constant(*ptr);
                    match self.globals.get(&name.to_string()) {
                        Some(v) => self.stack.push(v.clone()),
                        None => break runtime_error(format!("Symbol {} not found", name).as_str()),
                    }
                }
                OpCode::DefineLocal(ptr) => {
                    let name = chunk.read_constant(*ptr);
                    self.stack.push(Value::Symbol(name.to_string()));
                }
                OpCode::GetLocal(idx) => self.stack.push(self.stack[*idx].clone()),
                OpCode::Jump(ptr) => self.current_frame_mut().ip = *ptr,
                OpCode::JumpIfFalse(ptr) => {
                    let v = try!(self.peek());
                    if !v.truthy() {
                        self.current_frame_mut().ip = *ptr;
                    }
                }
                OpCode::Call(argc) => {
                    let f = try!(self.pick(*argc));
                    match f {
                        Value::Function(_, _) => {}
                        _ => break Err(format!("{} is not callable", f))
                    }
                    // TODO jump to bytecode for f
                }
                OpCode::Negate => {
                    let v = try!(self.pop());
                    let nv = try!(v.negate());
                    self.stack.push(nv);
                }
                OpCode::Add => {
                    let a = try!(self.pop());
                    let b = try!(self.pop());
                    let v = try!(b.add(&a));
                    self.stack.push(v);
                }
                OpCode::Subtract => {
                    let a = try!(self.pop());
                    let b = try!(self.pop());
                    let v = try!(b.subtract(&a));
                    self.stack.push(v);
                }
                OpCode::Multiply => {
                    let a = try!(self.pop());
                    let b = try!(self.pop());
                    let v = try!(b.multiply(&a));
                    self.stack.push(v);
                }
                OpCode::Divide => {
                    let a = try!(self.pop());
                    let b = try!(self.pop());
                    let v = try!(b.divide(&a));
                    self.stack.push(v);
                }
                OpCode::Not => {
                    let b = try!(self.pop());
                    self.stack.push(b.not());
                }
                OpCode::Equal => {
                    let a = try!(self.pop());
                    let b = try!(self.pop());
                    self.stack.push(b.equal(&a));
                }
                OpCode::GreaterThan => {
                    let a = try!(self.pop());
                    let b = try!(self.pop());
                    let v = try!(b.greater_than(&a));
                    self.stack.push(v);
                }
                OpCode::LessThan => {
                    let a = try!(self.pop());
                    let b = try!(self.pop());
                    let v = try!(b.less_than(&a));
                    self.stack.push(v);
                }
                OpCode::Print => {
                    let c = try!(self.pop());
                    println!("{}", c); // TODO raw print without newline
                    self.stack.push(Value::Nil);
                }
                OpCode::Pop => {
                    try!(self.pop());
                }
                OpCode::Zap(ptr) => {
                    if self.stack.len() <= *ptr {
                        return runtime_error("Zap out of bounds")
                    }
                    self.stack.remove(*ptr);
                }
                OpCode::Wipe => self.stack.clear(),
                OpCode::Return => {
                    let c = try!(self.pop());
                    println!("{}", c);
                }
            };
            self.current_frame_mut().ip += 1;
        }
    }
}

pub fn init_vm() -> VM {
    let top_frame = CallFrame{
        fn_name: String::from("main"),
        ip: 0,
        stack_start: 0,
    };
    VM{
        stack: vec![],
        globals: HashMap::new(),
        call_stack: vec![top_frame],
    }
}
