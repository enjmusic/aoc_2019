use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::path::PathBuf;
use structopt::StructOpt;
use std::collections::VecDeque;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short = "f", parse(from_os_str))]
    file: PathBuf,
}

#[derive(Copy, Clone, Debug)]
enum ParameterMode {
    Position,
    Immediate
}

#[derive(Clone, Debug)]
struct LoadParameter {
    param: i64,
    mode: ParameterMode,
}

impl LoadParameter {
    fn new(param: i64, mode: ParameterMode) -> LoadParameter {
        LoadParameter {
            param: param,
            mode: mode,
        }
    }
}

enum IntcodeInstruction {
    Exit,
    LoadInput { dest: usize },
    Output { val: LoadParameter },
    Add { o1: LoadParameter, o2: LoadParameter, dest: usize },
    Mul { o1: LoadParameter, o2: LoadParameter, dest: usize },
    LessThan { o1: LoadParameter, o2: LoadParameter, dest: usize },
    Equals { o1: LoadParameter, o2: LoadParameter, dest: usize },
    JumpIfTrue { predicate: LoadParameter, target: LoadParameter },
    JumpIfFalse { predicate: LoadParameter, target: LoadParameter },
}

trait IODevice {
    fn put(&mut self, output: i64);
    fn get(&mut self) -> Result<i64>;
}

struct DefaultInputDevice {
    buffer: VecDeque<i64>
}

struct DefaultOutputDevice {
    buffer: VecDeque<i64>
}

impl DefaultInputDevice {
    fn new() -> Box<DefaultInputDevice> {
        Box::new(DefaultInputDevice{ buffer: VecDeque::new() })
    }
}

impl DefaultOutputDevice {
    fn new() -> Box<DefaultOutputDevice> {
        Box::new(DefaultOutputDevice{ buffer: VecDeque::new() })
    }
}

impl IODevice for DefaultInputDevice {
    fn put(&mut self, output: i64) { self.buffer.push_front(output) }
    fn get(&mut self) -> Result<i64> {
        self.buffer.pop_back().map_or_else(|| {
            print!("Enter program input: ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            Ok(input.trim().parse::<i64>()?)
        }, |v| Ok(v))
    }
}

impl IODevice for DefaultOutputDevice {
    fn put(&mut self, output: i64) { self.buffer.push_front(output) }
    fn get(&mut self) -> Result<i64> {
        self.buffer.pop_back().map_or(Err(From::from("No output available")), |x| Ok(x))
    }
}

struct IntcodeProgram {
    memory: Vec<i64>,
    ip: usize,
    input: Box<dyn IODevice + Send>,
    output: Box<dyn IODevice + Send>,
}

fn instruction_param_length(opcode: u64) -> Result<usize> {
    match opcode {
        1 => Ok(3),
        2 => Ok(3),
        3 => Ok(1),
        4 => Ok(1),
        5 => Ok(2),
        6 => Ok(2),
        7 => Ok(3),
        8 => Ok(3),
        99 => Ok(0),
        _ => Err(From::from(format!("Invalid opcode: {}", opcode)))
    }
}

impl IntcodeProgram {
    fn from_raw_input(input: &String) -> Result<IntcodeProgram> {
        Ok(IntcodeProgram{
            memory: input.split(",").map(|item| {
                item.parse::<i64>().map_err(|_| From::from(format!("Invalid integer given: {}", item)))
            }).collect::<Result<Vec<i64>>>()?,
            ip: 0,
            input: DefaultInputDevice::new(),
            output: DefaultOutputDevice::new(),
        })
    }

    fn load_position(&self, location: usize) -> Result<i64> {
        if location >= self.memory.len() {
            return Err(From::from("Load location out of bounds"))
        }
        Ok(self.memory[location as usize])
    }

    fn load(&self, lp: LoadParameter) -> Result<i64> {
        match lp.mode {
            ParameterMode::Position => self.load_position(lp.param as usize),
            ParameterMode::Immediate => Ok(lp.param),
        }
    }

    fn store(&mut self, location: usize, value: i64) -> Result<()> {
        if location >= self.memory.len() {
            return Err(From::from("Load location out of bounds"))
        }
        self.memory[location] = value;
        Ok(())
    }

    // Returns the next instruction and increments the instruction
    // pointer to the subsequent yet-unfetched one, or returns error
    fn get_instruction(&mut self) -> Result<IntcodeInstruction> {
        let curr_ip = self.ip;
        let instruction = self.load_position(curr_ip)? as u64;
        let opcode = instruction % 100;
        let num_params = instruction_param_length(opcode)?;

        // This is cool but a little weird. We're making an infinite iterator that starts with
        // the provided parameter modes and is extended by the default (position) infinitely.
        let param_modes: Vec<ParameterMode> = (instruction / 100).to_string().chars().rev().map(|x| {
            if x == '0' { ParameterMode::Position } else { ParameterMode::Immediate }
        }).chain(std::iter::repeat(ParameterMode::Position)).take(num_params).collect();

        self.ip += 1 + num_params;
        match opcode {
            1 => {
                Ok(IntcodeInstruction::Add{
                    o1: LoadParameter::new(self.memory[curr_ip + 1], param_modes[0]),
                    o2: LoadParameter::new(self.memory[curr_ip + 2], param_modes[1]),
                    dest: self.memory[curr_ip + 3] as usize,
                })
            },
            2 => {
                Ok(IntcodeInstruction::Mul{
                    o1: LoadParameter::new(self.memory[curr_ip + 1], param_modes[0]),
                    o2: LoadParameter::new(self.memory[curr_ip + 2], param_modes[1]),
                    dest: self.memory[curr_ip + 3] as usize,
                })
            },
            3 => {
                Ok(IntcodeInstruction::LoadInput{
                    dest: self.memory[curr_ip + 1] as usize
                })
            },
            4 => {
                Ok(IntcodeInstruction::Output{
                    val: LoadParameter::new(self.memory[curr_ip + 1], param_modes[0]),
                })
            },
            5 => {
                Ok(IntcodeInstruction::JumpIfTrue{
                    predicate: LoadParameter::new(self.memory[curr_ip + 1], param_modes[0]),
                    target: LoadParameter::new(self.memory[curr_ip + 2], param_modes[1])
                })
            },
            6 => {
                Ok(IntcodeInstruction::JumpIfFalse{
                    predicate: LoadParameter::new(self.memory[curr_ip + 1], param_modes[0]),
                    target: LoadParameter::new(self.memory[curr_ip + 2], param_modes[1])
                })
            },
            7 => {
                Ok(IntcodeInstruction::LessThan{
                    o1: LoadParameter::new(self.memory[curr_ip + 1], param_modes[0]),
                    o2: LoadParameter::new(self.memory[curr_ip + 2], param_modes[1]),
                    dest: self.memory[curr_ip + 3] as usize,
                })
            },
            8 => {
                Ok(IntcodeInstruction::Equals{
                    o1: LoadParameter::new(self.memory[curr_ip + 1], param_modes[0]),
                    o2: LoadParameter::new(self.memory[curr_ip + 2], param_modes[1]),
                    dest: self.memory[curr_ip + 3] as usize,
                })
            },
            99 => Ok(IntcodeInstruction::Exit),
            _ => Err(From::from("Invalid opcode"))
        }
    }

    // Execute the next instruction at the instruction pointer, advancing
    // it and returning Ok(true) if the Intcode program should halt
    fn execute_next_instruction(&mut self) -> Result<bool> {
        match self.get_instruction()? {
            IntcodeInstruction::Exit => return Ok(true),
            IntcodeInstruction::LoadInput{dest} => {
                let input = self.input.get()?;
                self.store(dest, input)?;
            },
            IntcodeInstruction::Output{val} => {
                let output = self.load(val)?;
                self.output.put(output);
            },
            IntcodeInstruction::Add{o1, o2, dest} => {
                self.store(dest, self.load(o1)? + self.load(o2)?)?;
            },
            IntcodeInstruction::Mul{o1, o2, dest} => {
                self.store(dest, self.load(o1)? * self.load(o2)?)?;
            },
            IntcodeInstruction::JumpIfTrue{predicate, target} => {
                if self.load(predicate)? != 0 { self.ip = self.load(target)? as usize; }
            },
            IntcodeInstruction::JumpIfFalse{predicate, target} => {
                if self.load(predicate)? == 0 { self.ip = self.load(target)? as usize; }
            },
            IntcodeInstruction::LessThan{o1, o2, dest} => {
                self.store(dest, if self.load(o1)? < self.load(o2)? { 1 } else { 0 })?
            },
            IntcodeInstruction::Equals{o1, o2, dest} => {
                self.store(dest, if self.load(o1)? == self.load(o2)? { 1 } else { 0 })?
            }
        }

        Ok(false)
    }

    fn execute(&mut self) -> Result<()> {
        while !(self.execute_next_instruction()?) {}
        Ok(())
    }
}

fn main() -> Result<()> {
    let opt = Cli::from_args();

    let f = File::open(opt.file)?;
    let mut reader = BufReader::new(f);
    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;
    IntcodeProgram::from_raw_input(&contents)?.execute()
}

