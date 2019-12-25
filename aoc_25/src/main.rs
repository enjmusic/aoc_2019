use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::path::PathBuf;
use structopt::StructOpt;
use intcode::program::{Event, IntcodeProgram};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short = "f", parse(from_os_str))]
    file: PathBuf,
}

fn get_input_line(program: &mut IntcodeProgram) -> Result<()> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    for c in input.trim().chars() {
        program.give_input((c as u8) as i64);
    }
    program.give_input(10);
    Ok(())
}

fn run_text_adventure(input: &String) -> Result<()> {
    let mut program = IntcodeProgram::from_raw_input(input)?;
    let mut output_buffer = vec![];
    loop {
        match program.execute_until_event()? {
            Event::Exited => break,
            Event::InputRequired => get_input_line(&mut program)?,
            Event::ProducedOutput => {
                match program.get_output().unwrap() {
                    10 => {
                        println!("{}", output_buffer.iter().collect::<String>());
                        output_buffer.clear();
                    },
                    c => output_buffer.push((c as u8) as char)
                }
            },
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let opt = Cli::from_args();

    let f = File::open(opt.file)?;
    let mut reader = BufReader::new(f);
    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;
    
    // Weight required: food ration, space law space brochure, mutex, mouse, asterisk
    run_text_adventure(&contents)
}
