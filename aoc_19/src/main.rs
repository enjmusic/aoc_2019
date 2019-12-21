use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use structopt::StructOpt;
use intcode::program::{IntcodeProgram};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short = "f", parse(from_os_str))]
    file: PathBuf,
}

fn is_pulled(memory: &Vec<i64>, point: (usize, usize)) -> Result<bool> {
    let mut program = IntcodeProgram::from_memory(memory.to_vec());
    program.give_input(point.0 as i64);
    program.give_input(point.1 as i64);
    program.execute()?;
    program.get_output().map(|o| o != 0).ok_or(From::from("No output"))
}

fn part1(memory: &Vec<i64>) -> Result<()> {
    let mut num_pulled = 0;
    for y in 0..50 {
        for x in 0..50 {
            num_pulled += is_pulled(memory, (x, y))? as usize;
        }
    }
    Ok(println!("Number of 50x50 squares pulled by tractor beam: {}", num_pulled))
}

fn part2(memory: &Vec<i64>) -> Result<()> {
    let mut curr = (0, 50); // Make sure we're at a y with a decent width beam
    loop {
        while !is_pulled(memory, curr)? { curr.0 += 1; }
        if curr.0 >= 99 && curr.1 >= 99 {
            if is_pulled(memory, (curr.0 + 99, curr.1 - 99))? {
                return Ok(println!("Found square with top left edge at {}, {}", curr.0, curr.1 - 99))
            }
        }
        curr.1 += 1;
    }
}

fn main() -> Result<()> {
    let opt = Cli::from_args();

    let f = File::open(opt.file)?;
    let mut reader = BufReader::new(f);
    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;
    let memory = IntcodeProgram::raw_to_memory(&contents)?;

    part1(&memory)?;
    Ok(part2(&memory)?)
}
