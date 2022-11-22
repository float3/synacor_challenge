use std::io::Read;

//  - an unbounded stack which holds individual 16-bit values
//  - memory with 15-bit address space storing 16-bit values
//  - eight registers
#[derive(Debug)]

enum OpCode {
    Halt,
    Set(u16, u16),
    Push(u16),
    Pop(u16),
    Eq(u16, u16, u16),
    Gt(u16, u16, u16),
    Jmp(u16),
    Jt(u16, u16),
    Jf(u16, u16),
    Add(u16, u16, u16),
    Mult(u16, u16, u16),
    Mod(u16, u16, u16),
    And(u16, u16, u16),
    Or(u16, u16, u16),
    Not(u16, u16),
    Rmem(u16, u16),
    Wmem(u16, u16),
    Call(u16),
    Ret,
    Out(u16),
    In(u16),
    Noop,
    None,
}

pub struct Machine {
    memory: Vec<u16>,
    registers: Vec<u16>,
    stack: Vec<u16>,
    pos: u16,
}

impl Default for Machine {
    fn default() -> Self {
        Machine {
            memory: vec![0, 32768],
            registers: vec![0, 8],
            stack: vec![],
            pos: 0,
        }
    }
}

impl Machine {
    pub fn new() -> Machine {
        Machine {
            memory: vec![0; 32768],
            registers: vec![0; 8],
            stack: vec![],
            pos: 0,
        }
    }

    fn get_register(&self, reg: u16) -> u16 {
        self.registers[(reg - 32768) as usize]
    }

    fn set_register(&mut self, reg: u16, val: u16) {
        self.registers[(reg - 32768) as usize] = val;
    }

    fn value(&self, arg: u16) -> u16 {
        if arg <= 32767 {
            arg
        } else if arg <= 32775 {
            self.get_register(arg)
        } else {
            panic!()
        }
    }

    fn next(&mut self) -> u16 {
        let pos = self.pos as usize;
        let ret = self.memory[pos];
        self.pos = (pos + 1) as u16;
        ret
    }

    fn match_opcode(&mut self) -> OpCode {
        match self.next() {
            0 => OpCode::Halt,
            1 => OpCode::Set(self.next(), self.next()),
            2 => OpCode::Push(self.next()),
            3 => OpCode::Pop(self.next()),
            4 => OpCode::Eq(self.next(), self.next(), self.next()),
            5 => OpCode::Gt(self.next(), self.next(), self.next()),
            6 => OpCode::Jmp(self.next()),
            7 => OpCode::Jt(self.next(), self.next()),
            8 => OpCode::Jf(self.next(), self.next()),
            9 => OpCode::Add(self.next(), self.next(), self.next()),
            10 => OpCode::Mult(self.next(), self.next(), self.next()),
            11 => OpCode::Mod(self.next(), self.next(), self.next()),
            12 => OpCode::And(self.next(), self.next(), self.next()),
            13 => OpCode::Or(self.next(), self.next(), self.next()),
            14 => OpCode::Not(self.next(), self.next()),
            15 => OpCode::Rmem(self.next(), self.next()),
            16 => OpCode::Wmem(self.next(), self.next()),
            17 => OpCode::Call(self.next()),
            18 => OpCode::Ret,
            19 => OpCode::Out(self.next()),
            20 => OpCode::In(self.next()),
            21 => OpCode::Noop,
            _ => OpCode::None,
        }
    }

    fn add(&self, a: u16, b: u16) -> u16 {
        let res = ((a as u32) + (b as u32)) % 32768;
        res as u16
    }

    fn mult(&self, a: u16, b: u16) -> u16 {
        let res = ((a as u32) * (b as u32)) % 32768;
        res as u16
    }

    pub fn load(&mut self, path: &str) -> std::io::Result<u16> {
        use std::fs::File;
        use std::io::prelude::*;

        let mut file = File::open(path)?;
        let mut buf = vec![];
        let read = file.read_to_end(&mut buf)?;
        let mut i = 0;

        while i < buf.len() - 1 {
            self.memory[i / 2] = (buf[i] as u16) | ((buf[i + 1] as u16) << 8);
            i += 2;
        }

        Ok(read.try_into().unwrap())
    }

    pub fn tick(&mut self) -> bool {
        match self.match_opcode() {
            OpCode::Halt => {
                return false;
            }
            OpCode::Set(a, b) => {
                let val = self.value(b);
                self.set_register(a, val);
            }
            OpCode::Push(a) => {
                let val = self.value(a);
                self.stack.push(val);
            }
            OpCode::Pop(a) => {
                let top = self.stack.pop();
                match top {
                    Some(top_value) => self.set_register(a, top),
                    None => return false,
                }
            }
            OpCode::Eq(a, b, c) => {
                self.set_register(a, (self.value(b) == self.value(c)).into());
            }
            OpCode::Gt(a, b, c) => {
                self.set_register(a, (self.value(b) > self.value(c)).into());
            }
            OpCode::Jmp(a) => self.pos = self.value(a),
            OpCode::Jt(a, b) => {
                if self.value(a) != 0 {
                    self.pos = self.value(b)
                }
            }
            OpCode::Jf(a, b) => {
                if self.value(a) == 0 {
                    self.pos = self.value(b)
                }
            }
            OpCode::Add(a, b, c) => self.set_register(a, self.add(self.value(b), self.value(c))),
            OpCode::Mult(a, b, c) => self.set_register(a, self.mult(self.value(b), self.value(c))),
            OpCode::Mod(a, b, c) => self.set_register(a, self.value(b) % self.value(c)),
            OpCode::And(a, b, c) => self.set_register(a, self.value(b) & self.value(c)),
            OpCode::Or(a, b, c) => self.set_register(a, self.value(b) | self.value(c)),
            OpCode::Not(a, b) => self.set_register(a, (!self.value(b)) & 32767),
            OpCode::Rmem(a, b) => self.set_register(a, self.memory[self.value(b) as usize]),
            OpCode::Wmem(a, b) => {
                let address = self.value(a) as usize;
                self.memory[address] = self.value(b)
            }
            OpCode::Call(a) => {
                self.stack.push(self.pos);
                self.pos = self.value(a);
            }
            OpCode::Ret => match self.stack.pop() {
                Some(val) => self.pos = val,
                None => return false,
            },
            OpCode::Out(a) => print!("{}", (self.value(a) as u8) as char),
            OpCode::In(a) => {
                let c: u8 = std::io::stdin().bytes().nth(0).expect("EOF").expect("EOF");
                self.set_register(a, c as u16);
            }
            OpCode::Noop => (),

            _ => panic!(),
        }
        true
    }
    pub fn exec(&mut self) {
        while self.tick() {}
    }
}
