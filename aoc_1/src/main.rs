use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::process;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short = "r")]
    recurse: bool,
    #[structopt(short = "f", parse(from_os_str))]
    file: PathBuf,
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

fn main() {
    let opt = Cli::from_args();

    let file = File::open(opt.file.clone());
    let total_fuel_requirement: i64 = match file {
        Ok(f) => {
            let reader = BufReader::new(f);
            let mut accum_requirement: i64 = 0;

            for line in reader.lines() {
                let module_mass = line.unwrap().parse::<i64>().unwrap();
                accum_requirement += calc_fuel_requirement(module_mass, opt.recurse);
            }

            accum_requirement
        },
        Err(e) => {
            println!("Error opening file: {}", e);
            process::exit(1);
        }
    };

    println!("The total fuel requirement is: {}", total_fuel_requirement);
}
