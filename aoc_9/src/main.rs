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

fn main() -> Result<()> {
    let opt = Cli::from_args();

    let f = File::open(opt.file)?;
    let mut reader = BufReader::new(f);
    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;

    let mut program = IntcodeProgram::from_raw_input(&contents)?;
    program.execute()?;
    Ok(println!("{:?}", program.get_all_output()))
}

