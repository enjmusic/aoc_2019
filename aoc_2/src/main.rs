use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use structopt::StructOpt;
use intcode::program::IntcodeProgram;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short = "f", parse(from_os_str))]
    file: PathBuf,
    #[structopt(short = "d")]
    desired: Option<i64>,
    #[structopt(short = "n")]
    noun: Option<i64>,
    #[structopt(short = "v")]
    verb: Option<i64>,
}

fn run_intcode_for_noun_verb(memory: &Vec<i64>, noun: i64, verb: i64) -> Result<i64> {
    let mut new_memory = memory.clone();
    new_memory[1] = noun;
    new_memory[2] = verb;

    let mut program = IntcodeProgram::from_memory(new_memory);
    program.execute()?;
    Ok(program.load_position(0))
}

fn main() -> Result<()> {
    let opt = Cli::from_args();

    let f = File::open(opt.file)?;
    let mut reader = BufReader::new(f);
    let mut contents = String::new();
    reader.read_to_string(&mut contents).unwrap();

    let original_memory = IntcodeProgram::raw_to_memory(&contents)?;

    if let Some(desired_output) = opt.desired {
        // Scan noun/verb 0-99 to find desired output at location to examine
        for noun in 0..100 {
            for verb in 0..100 {
                if run_intcode_for_noun_verb(&original_memory, noun, verb)? == desired_output {
                    println!(
                        "Found values [noun: {}, verb: {}] that produce {} at location 0 after execution!",
                        noun,
                        verb,
                        desired_output,
                    );
                    println!("100 * noun + verb = {}", 100 * noun + verb);
                }
            }
        }
    } else if let (Some(noun), Some(verb)) = (opt.noun, opt.verb) {
        println!(
            "Value in memory location 0 after executing intcode: {}",
            run_intcode_for_noun_verb(&original_memory, noun, verb)?
        );
    } else {
        return Err(From::from("Was not provided with noun & verb or desired output!"))
    }

    Ok(())
}
