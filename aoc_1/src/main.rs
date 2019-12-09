use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use structopt::StructOpt;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short = "f", parse(from_os_str))]
    file: PathBuf,
    #[structopt(short = "r")]
    recurse: bool,
}

fn calc_fuel_requirement(mut mass: i64, should_recurse: bool) -> i64 {
    let mut accum: i64 = 0;

    loop {
        let curr_requirement: i64 = mass / 3 - 2;

        if curr_requirement <= 0 { break }

        accum += curr_requirement;
        mass = curr_requirement;

        if !should_recurse { break }
    }

    accum
}

fn main() -> Result<()> {
    let opt = Cli::from_args();
    let file = File::open(opt.file)?;
    let reader = BufReader::new(file);

    let module_masses = reader.lines().map(|line| {
        if let Ok(l) = line {
            l.parse::<i64>().map_err(|_| From::from("Failed to parse line"))
        } else {
            Err(From::from("Failed to read line"))
        }
    }).collect::<Result<Vec<i64>>>()?;

    let recurse = opt.recurse;
    let total_fuel_requirement = module_masses.iter()
        .fold(0, |acc, mass| acc + calc_fuel_requirement(*mass, recurse));

    Ok(println!("The total fuel requirement is: {}", total_fuel_requirement))
}
