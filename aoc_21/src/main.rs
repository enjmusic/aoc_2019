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
}

fn input_line(program: &mut IntcodeProgram, line: &str) {
    for c in line.chars() {
        program.give_input(c as i64);
    }
    program.give_input(10);
}

#[allow(dead_code)]
fn print_ascii_output(program: &mut IntcodeProgram) {
    for c in program.get_all_output() {
        match c {
            10 => println!(""),
            _ => print!("{}", (c as u8) as char)
        }
    }
}

fn part1(input: &String) -> Result<()> {
    let mut program = IntcodeProgram::from_raw_input(input)?;
    let swift_script = vec![
        // !c && d
        "NOT C T",
        "AND D T",
        "OR T J",

        // !a
        "NOT A T",
        "OR T J",

        "WALK",
    ];

    for line in swift_script {
        input_line(&mut program, line);
    }
    program.execute()?;
    Ok(println!("Hull damage: {}", program.get_all_output().last().unwrap_or(&0)))
}

fn part2(input: &String) -> Result<()> {
    let mut program = IntcodeProgram::from_raw_input(input)?;
    let swift_script = vec![
        // !c && d && (!f || h)
        "NOT C T",
        "AND D T",
        "OR T J",
        "NOT F T",
        "OR H T",
        "AND T J",

        // !a
        "NOT A T",
        "OR T J",

        // a && !b
        "NOT B T",
        "AND A T",
        "AND D T",
        "OR T J",

        "RUN"
    ];

    for line in swift_script {
        input_line(&mut program, line);
    }
    program.execute()?;
    Ok(println!("Hull damage: {}", program.get_all_output().last().unwrap_or(&0)))
}

fn main() -> Result<()> {
    let opt = Cli::from_args();

    let f = File::open(opt.file)?;
    let mut reader = BufReader::new(f);
    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;

    part1(&contents)?;
    part2(&contents)?;
    Ok(())
}