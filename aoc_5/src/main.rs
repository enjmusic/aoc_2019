use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::path::PathBuf;
use structopt::StructOpt;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short = "f", parse(from_os_str))]
    file: PathBuf,
}

#[derive(Copy, Clone)]
enum ParameterMode {
    Position,
    Immediate
}

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
}

struct IntcodeProgram {
    memory: Vec<i64>,
    ip: usize,
}

fn input_to_memory(input: &String) -> Result<Vec<i64>> {
    input.split(",").map(|item| {
        if let Ok(integer) = item.parse::<i64>() {
            Ok(integer)
        } else {
            Err(From::from("Invalid integer given"))
        }
    }).collect()
}

fn digits(i: u64) -> Result<Vec<u8>> {
    i.to_string().chars().map(|x| {
        x.to_digit(10).ok_or(From::from(format!("Invalid digit {}", x))).map(|i| i as u8)
    }).collect()
}

fn get_int() -> Result<i64> {
    print!("Enter program input: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().parse::<i64>()?)
}

impl IntcodeProgram {
    fn from_raw_input(input: &String) -> Result<IntcodeProgram> {
        Ok(IntcodeProgram{
            memory: input_to_memory(input)?,
            ip: 0,
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
        let instruction = self.load_position(self.ip)? as u64;
        let opcode = instruction % 100;

        // This is cool but a little hard to look at. We're making
        // an infinite iterator that starts with the provided parameter
        // modes and is extended by the default (position) infinitely.
        let param_mode_digits = digits(instruction / 100)?;
        let param_modes_iter = param_mode_digits.iter().rev().map(|&i| {
            if i == 0 { ParameterMode::Position } else { ParameterMode::Immediate }
        }).chain(std::iter::repeat(ParameterMode::Position));

        // Make parsed instruction and amount to advance instruction pointer
        let parsed = match opcode {
            1 => {
                let modes: Vec<ParameterMode> = param_modes_iter.take(3).collect();
                (Ok(IntcodeInstruction::Add{
                    o1: LoadParameter::new(self.memory[self.ip + 1], modes[0]),
                    o2: LoadParameter::new(self.memory[self.ip + 2], modes[1]),
                    dest: self.memory[self.ip + 3] as usize,
                }), 4)
            },
            2 => {
                let modes: Vec<ParameterMode> = param_modes_iter.take(3).collect();
                (Ok(IntcodeInstruction::Mul{
                    o1: LoadParameter::new(self.memory[self.ip + 1], modes[0]),
                    o2: LoadParameter::new(self.memory[self.ip + 2], modes[1]),
                    dest: self.memory[self.ip + 3] as usize,
                }), 4)
            },
            3 => {
                (Ok(IntcodeInstruction::LoadInput{
                    dest: self.memory[self.ip + 1] as usize
                }), 2)
            },
            4 => {
                let modes: Vec<ParameterMode> = param_modes_iter.take(1).collect();
                (Ok(IntcodeInstruction::Output{
                    val: LoadParameter::new(self.memory[self.ip + 1], modes[0]),
                }), 2)
            },
            99 => (Ok(IntcodeInstruction::Exit), 1),
            _ => (Err(From::from("Invalid opcode")), 0)
        };

        self.ip += parsed.1;
        parsed.0
    }

    // Execute the next instruction at the instruction pointer, advancing
    // it and returning Ok(true) if the Intcode program should halt
    fn execute_next_instruction(&mut self) -> Result<bool> {
        match self.get_instruction()? {
            IntcodeInstruction::Exit => Ok(true),
            IntcodeInstruction::LoadInput{dest} => {
                self.store(dest, get_int()?)?;
                Ok(false)
            },
            IntcodeInstruction::Output{val} => {
                println!("Program output: {}", self.load(val)?);
                Ok(false)
            },
            IntcodeInstruction::Add{o1, o2, dest} => {
                self.store(dest, self.load(o1)? + self.load(o2)?)?;
                Ok(false)
            },
            IntcodeInstruction::Mul{o1, o2, dest} => {
                self.store(dest, self.load(o1)? * self.load(o2)?)?;
                Ok(false)
            },
        }
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
    reader.read_to_string(&mut contents).unwrap();

    IntcodeProgram::from_raw_input(&contents)?.execute()
}
