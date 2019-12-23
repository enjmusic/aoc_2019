use super::io;
use std::collections::HashMap;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(PartialEq)]
pub enum Event {
    InputRequired,
    ProducedOutput,
    Exited,
}

#[derive(Copy, Clone, Debug)]
enum ParameterMode {
    Position,
    Immediate,
    Relative
}

#[derive(Clone, Debug)]
struct Parameter {
    param: i64,
    mode: ParameterMode,
}

impl Parameter {
    fn new(param: i64, mode: ParameterMode) -> Parameter {
        Parameter {
            param: param,
            mode: mode,
        }
    }
}

impl std::fmt::Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.mode {
            ParameterMode::Position => write!(f, "[{}]", self.param),
            ParameterMode::Immediate => write!(f, "{}", self.param),
            ParameterMode::Relative => write!(f, "[rb + {}]", self.param),
        }
    }
}

#[derive(Debug)]
enum IntcodeInstruction {
    Add { o1: Parameter, o2: Parameter, dest: Parameter },
    Mul { o1: Parameter, o2: Parameter, dest: Parameter },
    LoadInput { dest: Parameter },
    Output { val: Parameter },
    LessThan { o1: Parameter, o2: Parameter, dest: Parameter },
    Equals { o1: Parameter, o2: Parameter, dest: Parameter },
    JumpIfTrue { predicate: Parameter, target: Parameter },
    JumpIfFalse { predicate: Parameter, target: Parameter },
    AdjustRelativeBase { val: Parameter },
    Exit,
}

impl std::fmt::Display for IntcodeInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            IntcodeInstruction::Add{o1, o2, dest} => {
                write!(f, "add: {} <- {} + {}", dest, o1, o2)
            },
            IntcodeInstruction::Mul{o1, o2, dest} => {
                write!(f, "mul: {} <- {} + {}", dest, o1, o2)
            },
            IntcodeInstruction::LoadInput{dest} => {
                write!(f, "in: {}", dest)
            },
            IntcodeInstruction::Output{val} => {
                write!(f, "out: {}", val)
            },
            IntcodeInstruction::JumpIfTrue{predicate, target} => {
                write!(f, "jnz: {} if {}", target, predicate)
            },
            IntcodeInstruction::JumpIfFalse{predicate, target} => {
                write!(f, "jez: {} if not {}", target, predicate)
            },
            IntcodeInstruction::LessThan{o1, o2, dest} => {
                write!(f, "lt: {} <- {} < {}", dest, o1, o2)
            },
            IntcodeInstruction::Equals{o1, o2, dest} => {
                write!(f, "eq: {} <- {} == {}", dest, o1, o2)
            },
            IntcodeInstruction::AdjustRelativeBase{val} => {
                write!(f, "arb {}", val)
            },
            IntcodeInstruction::Exit => write!(f, "hlt"),
        }
    }
}

enum Assembly {
    Data(i64),
    Instruction(Vec<i64>, IntcodeInstruction),
}

impl std::fmt::Display for Assembly {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Assembly::Data(data) => write!(f, "{:<30} ; data", data),
            Assembly::Instruction(raw, instr) => write!(f, "{:<30} ; {}", format!("{:?}", raw), instr),
        }
    }
}

pub struct IntcodeProgram {
    memory: Vec<i64>,
    extended_memory: HashMap<usize, i64>,
    ip: usize,
    relative_base: i64,
    input: Box<dyn io::InputDevice + Send>,
    output: Box<dyn io::OutputDevice + Send>,
}

fn instruction_param_length(opcode: i64) -> Result<usize> {
    match opcode {
        1 => Ok(3),
        2 => Ok(3),
        3 => Ok(1),
        4 => Ok(1),
        5 => Ok(2),
        6 => Ok(2),
        7 => Ok(3),
        8 => Ok(3),
        9 => Ok(1),
        99 => Ok(0),
        _ => Err(From::from(format!("Invalid opcode: {}", opcode)))
    }
}

impl IntcodeProgram {
    pub fn raw_to_memory(raw: &String) -> Result<Vec<i64>> {
        raw.split(",").map(|item| {
            item.parse::<i64>().map_err(|_| {
                From::from(format!("Invalid integer given: {}", item))
            })
        }).collect()
    }

    pub fn from_raw_input(input: &String) -> Result<IntcodeProgram> {
        Ok(IntcodeProgram::from_memory(
            IntcodeProgram::raw_to_memory(input)?
        ))
    }

    pub fn from_memory(memory: Vec<i64>) -> IntcodeProgram {
        IntcodeProgram{
            memory: memory,
            extended_memory: HashMap::new(),
            ip: 0,
            relative_base: 0,
            input: io::DefaultInputDevice::new(),
            output: io::DefaultOutputDevice::new(),
        }
    }

    pub fn load_position(&self, location: usize) -> i64 {
        if location >= self.memory.len() {
            *self.extended_memory.get(&location).unwrap_or(&0)
        } else {
            self.memory[location as usize]
        }
    }

    fn load(&self, p: Parameter) -> i64 {
        match p.mode {
            ParameterMode::Position => self.load_position(p.param as usize),
            ParameterMode::Immediate => p.param,
            ParameterMode::Relative => self.load_position((p.param + self.relative_base) as usize)
        }
    }

    fn store(&mut self, p: Parameter, value: i64) {
        let location = match p.mode {
            ParameterMode::Relative => (p.param + self.relative_base) as usize,
            _ => p.param as usize,
        };

        if location >= self.memory.len() {
            self.extended_memory.insert(location, value);
        } else {
            self.memory[location] = value;
        }
    }

    // Returns the next instruction and increments the instruction
    // pointer to the subsequent yet-unfetched one, or returns error
    fn get_instruction(&mut self) -> Result<IntcodeInstruction> {
        let curr_ip = self.ip;
        let instruction = self.load_position(curr_ip);
        let opcode = instruction % 100;
        let num_params = instruction_param_length(opcode)?;

        // This is cool but a little weird. We're making an infinite iterator that starts with
        // the provided parameter modes and is extended by the default (position) infinitely.
        let param_modes: Vec<ParameterMode> = (instruction / 100).to_string().chars().rev().map(|x| {
            match x {
                '0' => ParameterMode::Position,
                '1' => ParameterMode::Immediate,
                _ => ParameterMode::Relative,
            }
        }).chain(std::iter::repeat(ParameterMode::Position)).take(num_params).collect();

        self.ip += 1 + num_params;
        match opcode {
            1 => {
                Ok(IntcodeInstruction::Add{
                    o1: Parameter::new(self.load_position(curr_ip + 1), param_modes[0]),
                    o2: Parameter::new(self.load_position(curr_ip + 2), param_modes[1]),
                    dest: Parameter::new(self.load_position(curr_ip + 3), param_modes[2]),
                })
            },
            2 => {
                Ok(IntcodeInstruction::Mul{
                    o1: Parameter::new(self.load_position(curr_ip + 1), param_modes[0]),
                    o2: Parameter::new(self.load_position(curr_ip + 2), param_modes[1]),
                    dest: Parameter::new(self.load_position(curr_ip + 3), param_modes[2]),
                })
            },
            3 => {
                Ok(IntcodeInstruction::LoadInput{
                    dest: Parameter::new(self.load_position(curr_ip + 1), param_modes[0]),
                })
            },
            4 => {
                Ok(IntcodeInstruction::Output{
                    val: Parameter::new(self.load_position(curr_ip + 1), param_modes[0]),
                })
            },
            5 => {
                Ok(IntcodeInstruction::JumpIfTrue{
                    predicate: Parameter::new(self.load_position(curr_ip + 1), param_modes[0]),
                    target: Parameter::new(self.load_position(curr_ip + 2), param_modes[1])
                })
            },
            6 => {
                Ok(IntcodeInstruction::JumpIfFalse{
                    predicate: Parameter::new(self.load_position(curr_ip + 1), param_modes[0]),
                    target: Parameter::new(self.load_position(curr_ip + 2), param_modes[1])
                })
            },
            7 => {
                Ok(IntcodeInstruction::LessThan{
                    o1: Parameter::new(self.load_position(curr_ip + 1), param_modes[0]),
                    o2: Parameter::new(self.load_position(curr_ip + 2), param_modes[1]),
                    dest: Parameter::new(self.load_position(curr_ip + 3), param_modes[2]),
                })
            },
            8 => {
                Ok(IntcodeInstruction::Equals{
                    o1: Parameter::new(self.load_position(curr_ip + 1), param_modes[0]),
                    o2: Parameter::new(self.load_position(curr_ip + 2), param_modes[1]),
                    dest: Parameter::new(self.load_position(curr_ip + 3), param_modes[2]),
                })
            },
            9 => {
                Ok(IntcodeInstruction::AdjustRelativeBase{
                    val: Parameter::new(self.load_position(curr_ip + 1), param_modes[0]),
                })
            }
            99 => Ok(IntcodeInstruction::Exit),
            _ => Err(From::from("Invalid opcode"))
        }
    }

    // Execute the next instruction at the instruction pointer, advancing
    // it and returning Ok(true) if the Intcode program should halt
    fn execute_instruction(&mut self, instruction: IntcodeInstruction, input_break: bool) -> Result<Option<Event>> {
        match instruction {
            IntcodeInstruction::Add{o1, o2, dest} => {
                self.store(dest, self.load(o1) + self.load(o2));
            },
            IntcodeInstruction::Mul{o1, o2, dest} => {
                self.store(dest, self.load(o1) * self.load(o2));
            },
            IntcodeInstruction::LoadInput{dest} => {
                if input_break {
                    if let Some(input) = self.input.get_maybe() {
                        self.store(dest, input);
                    } else {
                        return Ok(Some(Event::InputRequired))
                    }
                } else {
                    let input = self.input.get()?;
                    self.store(dest, input);
                }
            },
            IntcodeInstruction::Output{val} => {
                let output = self.load(val);
                self.output.put(output);
                return Ok(Some(Event::ProducedOutput))
            },
            IntcodeInstruction::JumpIfTrue{predicate, target} => {
                if self.load(predicate) != 0 { self.ip = self.load(target) as usize; }
            },
            IntcodeInstruction::JumpIfFalse{predicate, target} => {
                if self.load(predicate) == 0 { self.ip = self.load(target) as usize; }
            },
            IntcodeInstruction::LessThan{o1, o2, dest} => {
                self.store(dest, if self.load(o1) < self.load(o2) { 1 } else { 0 })
            },
            IntcodeInstruction::Equals{o1, o2, dest} => {
                self.store(dest, if self.load(o1) == self.load(o2) { 1 } else { 0 })
            },
            IntcodeInstruction::AdjustRelativeBase{val} => {
                self.relative_base += self.load(val);
            },
            IntcodeInstruction::Exit => return Ok(Some(Event::Exited)),
        }

        Ok(None)
    }

    pub fn execute(&mut self) -> Result<()> {
        loop {
            let instruction = self.get_instruction()?;
            if let Some(Event::Exited) = self.execute_instruction(instruction, false)? { return Ok(()) }
        }
    }

    pub fn execute_until_event(&mut self) -> Result<Event> {
        loop {
            let curr_ip = self.ip;
            let instruction = self.get_instruction()?;
            if let Some(event) = self.execute_instruction(instruction, true)? {
                match event {
                    Event::ProducedOutput => (),
                    _ => self.ip = curr_ip // Keep program at same instruction for input/exit
                }
                return Ok(event)
            }
        }
    }

    pub fn replace_input(&mut self, new: Box<dyn io::InputDevice + Send>) {
        self.input = new;
    }

    pub fn replace_output(&mut self, new: Box<dyn io::OutputDevice + Send>) {
        self.output = new;
    }

    pub fn give_input(&mut self, input: i64) { self.input.put(input) }
    pub fn get_output(&mut self) -> Option<i64> { self.output.get() }
    pub fn get_all_output(&mut self) -> Vec<i64> {
        std::iter::repeat_with(|| self.output.get())
            .take_while(|o| o.is_some()).map(|o| o.unwrap()).collect()
    }

    pub fn disassemble(&mut self) {
        std::iter::repeat_with(|| {
            let curr_ip = self.ip;
            let curr_opcode = self.load_position(curr_ip) % 100;
            if self.ip >= self.memory.len() {
                None
            } else if let Ok(instr) = self.get_instruction() {
                Some((curr_ip, Assembly::Instruction(
                    (curr_ip..(curr_ip + 1 + instruction_param_length(curr_opcode).unwrap()))
                        .map(|i| self.memory[i]).collect(),
                    instr
                )))
            } else {
                self.ip += 1;
                Some((curr_ip, Assembly::Data(self.load_position(curr_ip))))
            }
        }).take_while(|a| a.is_some()).map(|a| a.unwrap()).for_each(|(ip, a)| println!("{:>10} : {}", ip, a));
    }
}