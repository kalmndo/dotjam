#[derive(Debug, PartialEq)]
pub enum ExitReason {
    Panic,
    OutOfGas,
    Halt(u32),
    HostCall,
    Fault(u32)
}

pub struct Machine {
    registers: [u64; 13],
    pc: u32,
    memory: Vec<u8>,
    gas: u64,
    exit: Option<ExitReason>,
    c: Vec<u8>,
    k: Vec<u8>,
    j: Vec<u32>
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
            j: Vec::new()
        }
    }

    pub fn load_program(&mut self, c: Vec<u8>, k: Vec<u8>, j: Vec<u32>) {
        self.c = c;
        self.k = k;
        self.j = j;
    }

    pub fn step(&mut self) -> Option<ExitReason> {
        // gate 1: gas
        if self.gas == 0 {
            return Some(ExitReason::OutOfGas);
        }
        // gate 2: memory
        // gate 3: execute
        let opcode = self.c[self.pc as usize];

        match opcode {
            0 => return Some(ExitReason::Panic),
            1 => { self.pc += 1; return None },
            _ => return Some(ExitReason::Panic)
        }
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
}
