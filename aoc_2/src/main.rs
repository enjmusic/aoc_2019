use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::process;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short = "a", default_value = "0")]
    output_address: usize,
    #[structopt(short = "f", parse(from_os_str))]
    file: PathBuf,
    #[structopt(short = "d")]
    desired: Option<i64>,
    #[structopt(short = "n")]
    noun: Option<i64>,
    #[structopt(short = "v")]
    verb: Option<i64>,
}

enum Instruction {
    Add,
    Multiply,
    Exit
}

fn opcode_to_instruction(opcode: i64) -> Option<Instruction> {
    match opcode {
        1 => Some(Instruction::Add),
        2 => Some(Instruction::Multiply),
        99 => Some(Instruction::Exit),
        _ => None
    }
}

fn execute_intcode(mut memory: Vec<i64>, output_addr: usize) -> i64 {
    let mut instruction_pointer: usize = 0;

    loop {
        let opcode_number = memory[instruction_pointer];
        match opcode_to_instruction(opcode_number) {
            None => {
                println!("Unknown opcode {} - aborting program!", opcode_number);
                process::exit(1);
            },
            Some(Instruction::Exit) => break,
            Some(binary_op) => {
                let operand1_addr = memory[instruction_pointer + 1] as usize;
                let operand2_addr = memory[instruction_pointer + 2] as usize;
                let dest_addr = memory[instruction_pointer + 3] as usize;

                match binary_op {
                    Instruction::Add => memory[dest_addr] = memory[operand1_addr] + memory[operand2_addr],
                    _ => memory[dest_addr] = memory[operand1_addr] * memory[operand2_addr],
                }
            }
        }

        instruction_pointer += 4;
        if instruction_pointer >= memory.len() {
            println!(
                "Instruction pointer at {} ran over end of memory (length {}) - aborting program!",
                instruction_pointer,
                memory.len()
            );
            process::exit(1);
        }
    }

    memory[output_addr]
}

fn main() {
    let opt = Cli::from_args();

    let f = File::open(opt.file.clone());
    if let Err(e) = f {
        println!("Failed to open input file: {}", e);
        process::exit(1);
    }
    
    let mut reader = BufReader::new(f.unwrap());
    let mut contents = String::new();
    reader.read_to_string(&mut contents).unwrap();

    let memory: Vec<i64> = contents.split(",").map(|item| item.parse::<i64>().unwrap()).collect();

    if opt.output_address >= memory.len() {
        println!("Will not attempt to calculate nonexistent memory location: {}", opt.output_address);
        process::exit(1);
    }

    if let Some(desired_output) = opt.desired {
        // Scan noun/verb 0-99 to find desired output at location to examine
        for noun in 0..100 {
            for verb in 0..100 {
                let mut new_memory = memory.clone();
                new_memory[1] = noun;
                new_memory[2] = verb;

                let output = execute_intcode(new_memory, opt.output_address);
                if output == desired_output {
                    println!(
                        "Found values [noun: {}, verb: {}] that produce {} at location {} after execution!",
                        noun,
                        verb,
                        desired_output,
                        opt.output_address
                    );
                    println!("100 * noun + verb = {}", 100 * noun + verb);
                }
            }
        }
    } else if let (Some(noun), Some(verb)) = (opt.noun, opt.verb) {
        // Calculate value at location to examine with given input
        let mut new_memory = memory.clone();
        new_memory[1] = noun;
        new_memory[2] = verb;

        let output = execute_intcode(new_memory, opt.output_address);
        println!(
            "Value in memory location {} after executing intcode: {}",
            opt.output_address,
            output
        );
    } else {
        println!("Was not provided with noun & verb or desired output!");
        process::exit(1);
    }
}
