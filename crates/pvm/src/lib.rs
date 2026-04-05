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
    gas: i64,
    gas_charged: bool,
    exit: Option<ExitReason>,
    c: Vec<u8>,
    k: Vec<u8>,
    j: Vec<u32>,
}

impl Machine {
    pub fn new(gas: i64) -> Self {
        Machine {
            registers: [0u64; 13],
            pc: 0,
            memory: Vec::new(),
            gas,
            gas_charged: false,
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
        if !self.gas_charged {
            let block_cost: i64 = 1; // TODO: Think about this calculation later
            if self.gas >= block_cost {
                self.gas -= block_cost;
                self.gas_charged = true
            } else {
                return Some(ExitReason::OutOfGas);
            }
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
            100 => {
                let r_d = self.c[(self.pc + 1) as usize] % 16;
                let r_a = self.c[(self.pc + 1) as usize] / 16;
                self.registers[r_d as usize] = self.registers[r_a as usize];
                self.pc = next_pc;
                return None;
            }
            170 => {
                let r_a = self.c[(self.pc + 1) as usize] % 16;
                let r_b = self.c[(self.pc + 1) as usize] / 16;
                let offset = self.c[(self.pc + 2) as usize];

                if self.registers[r_a as usize] == self.registers[r_b as usize] {
                    self.pc += offset as u32;
                } else {
                    self.pc = next_pc;
                }
                self.gas_charged = false;

                return None;
            }
            191 => {
                let r_a = self.c[(self.pc + 1) as usize] % 16;
                let r_b = self.c[(self.pc + 1) as usize] / 16;
                let r_d = self.c[(self.pc + 2) as usize];

                self.registers[r_d as usize] =
                    self.registers[r_a as usize].wrapping_sub(self.registers[r_b as usize]);
                self.pc = next_pc;
                return None;
            }
            192 => {
                let r_a = self.c[(self.pc + 1) as usize] % 16;
                let r_b = self.c[(self.pc + 1) as usize] / 16;
                let r_d = self.c[(self.pc + 2) as usize];

                self.registers[r_d as usize] =
                    self.registers[r_a as usize].wrapping_mul(self.registers[r_b as usize]);
                self.pc = next_pc;
                return None;
            }
            200 => {
                let r_a = self.c[(self.pc + 1) as usize] % 16;
                let r_b = self.c[(self.pc + 1) as usize] / 16;
                let r_d = self.c[(self.pc + 2) as usize];

                self.registers[r_d as usize] =
                    self.registers[r_a as usize] + self.registers[r_b as usize];
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

    fn run(&mut self) -> ExitReason {
        loop {
            if let Some(reason) = self.step() {
                return reason;
            }
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
        let result = m.run();
        assert_eq!(result, ExitReason::Panic);
    }

    #[test]
    fn load_imm_loads_value() {
        let mut m = Machine::new(1000);
        m.load_program(vec![51, 0x00, 42, 0], vec![1, 0, 0, 1], vec![]);
        m.run();
        assert_eq!(m.register(0), 42)
    }

    #[test]
    fn move_reg() {
        let mut m = Machine::new(1000);
        m.load_program(
            vec![51, 0x00, 42, 100, 0x01, 0],
            vec![1, 0, 0, 1, 0, 1],
            vec![],
        );
        m.run();
        assert_eq!(m.register(1), 42)
    }

    #[test]
    fn add_two_register_store_third() {
        let mut m = Machine::new(1000);
        m.load_program(
            vec![51, 0x00, 42, 51, 0x01, 3, 200, 0x01, 0x03, 0],
            vec![1, 0, 0, 1, 0, 0, 1, 0, 0, 1],
            vec![],
        );
        m.run();
        assert_eq!(m.register(3), 45)
    }

    #[test]
    fn sub_32() {
        let mut m = Machine::new(1000);
        m.load_program(
            vec![51, 0x00, 10, 51, 0x01, 3, 191, 0x10, 0x02, 0],
            vec![1, 0, 0, 1, 0, 0, 1, 0, 0, 1],
            vec![],
        );
        m.run();
        assert_eq!(m.register(2), 7)
    }

    #[test]
    fn mul_32() {
        let mut m = Machine::new(1000);
        m.load_program(
            vec![51, 0x00, 5, 51, 0x01, 3, 192, 0x10, 0x02, 0],
            vec![1, 0, 0, 1, 0, 0, 1, 0, 0, 1],
            vec![],
        );
        m.run();
        assert_eq!(m.register(2), 15)
    }

    #[test]
    fn branch_eq_jumps_when_equal() {
        let mut m = Machine::new(1000);
        m.load_program(
            vec![51, 0x00, 5, 51, 0x01, 5, 170, 0x10, 4, 0, 51, 0x02, 99, 0],
            vec![1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 1, 0, 0, 1],
            vec![],
        );
        m.run();
        assert_eq!(m.register(2), 99);
    }

    #[test]
    fn should_out_of_gas() {
        let mut m = Machine::new(1);
        m.load_program(
            vec![
                51, 0x00, 5, 51, 0x01, 5, 170, 0x10, 4, 0, 51, 0x02, 99, 51, 0x00, 5, 51, 0x01, 5,
                0,
            ],
            vec![1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1],
            vec![],
        );
        let result = m.run();
        assert_eq!(result, ExitReason::OutOfGas)
    }
}
