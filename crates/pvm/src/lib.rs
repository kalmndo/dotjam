#[derive(Debug, PartialEq)]
pub enum ExitReason {
    Panic,
    OutOfGas,
    Halt(u32),
    HostCall,
    Fault(u32),
}

pub struct Machine {
    registers: [u64; 13],
    pc: u32,
    memory: Vec<u8>,
    gas: u64,
    exit: Option<ExitReason>,
    c: Vec<u8>,
    k: Vec<u8>,
    j: Vec<u32>,
}

impl Machine {
    pub fn new(gas: u64) -> Self {
        Machine {
            registers: [0u64; 13],
            pc: 0,
            memory: Vec::new(),
            gas,
            exit: None,
            c: Vec::new(),
            k: Vec::new(),
            j: Vec::new(),
        }
    }

    pub fn load_program(&mut self, c: Vec<u8>, k: Vec<u8>, j: Vec<u32>) {
        self.c = c;
        self.k = k;
        self.j = j;
    }

    pub fn register(&self, index: usize) -> u64 {
        self.registers[index]
    }

    pub fn step(&mut self) -> Option<ExitReason> {
        // gate 1: gas
        if self.gas == 0 {
            return Some(ExitReason::OutOfGas);
        }
        // gate 2: memory
        // gate 3: execute
        let opcode = self.c[self.pc as usize];

        let skip = self.skip(self.pc);
        let next_pc = self.pc + 1 + skip;

        match opcode {
            0 => return Some(ExitReason::Panic),
            1 => {
                self.pc = next_pc;
                return None;
            }
            51 => {
                let r_a = self.c[(self.pc + 1) as usize] % 16; // lower nibble = register index
                let num_bytes = skip - 1;
                let mut value = 0;

                for i in 0..num_bytes {
                    value |= (self.c[(self.pc + 2 + i) as usize] as u64) << (8 * i)
                }

                self.registers[r_a as usize] = value;
                self.pc = next_pc;
                return None;
            }
            _ => return Some(ExitReason::Panic),
        }
    }

    fn skip(&self, pc: u32) -> u32 {
        let start = pc as usize + 1;
        let mut i = start;
        while i < self.k.len() && self.k[i] == 0 {
            i += 1
        }
        (i - start) as u32
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn trap_returns_panic() {
        let mut m = Machine::new(1000);
        m.load_program(vec![0], vec![1], vec![]);
        let result = m.step();
        assert_eq!(result, Some(ExitReason::Panic));
    }

    #[test]
    fn load_imm_loads_value() {
        let mut m = Machine::new(1000);
        m.load_program(vec![51, 0x00, 42], vec![1, 0, 0], vec![]);
        m.step();
        assert_eq!(m.register(0), 42)
    }
}
