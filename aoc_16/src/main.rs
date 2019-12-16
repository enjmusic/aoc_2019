use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use structopt::StructOpt;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short = "f", parse(from_os_str))]
    file: PathBuf,
}

fn apply_fft(vals: &Vec<i64>) -> Vec<i64> {
    let mut out = vec![0; vals.len()];

    for i in 0..vals.len() {
        let mut sum = 0;
        for j in i..vals.len() { // Skip zeroes in upper-diagonal part of transform
            let phase = ((j + 1) / (i + 1)) & 3; // 0 -> 0, 1 -> 1, 2 -> -1, 3 -> 0
            if phase & 1 != 0 { sum += if phase == 1 { vals[j] } else { -vals[j] }; }
        }
        out[i] = sum.abs() % 10;
    }

    out
}

fn part1(vals: &Vec<i64>) {
    let mut input = vals.clone();
    for _ in 0..100 {
        input = apply_fft(&input);
    }
    println!(
        "First 8 digits after 100 FFTs: {}",
        input.iter().take(8).map(|x| x.to_string()).collect::<String>()
    );
}

fn part2(vals: &Vec<i64>) {
    let index = vals.iter().take(7).fold(0, |acc, val| val + acc * 10) as usize;
    if index < vals.len() * 10000 / 2 {
        println!("Index {} not in predictable part of transformation matrix. Forget about it.", index);
        return
    }

    // Because of how the patterns work for the 2nd half of the values, everything under
    // the diagonal of the transformation matrix is a one. So, for indices past halfway
    // their next FFT is just the last digit of the sum of all digits after them. We can
    // construct each one of these in O(n) time by maintaining a partial sum over time.
    let mut relevant_values: Vec<i64> = (index..(vals.len() * 10000)).map(|i| vals[i % vals.len()]).collect::<Vec<i64>>();
    for _ in 0..100 {
        let mut partial_sum = 0;
        for i in (0..relevant_values.len()).rev() {
            partial_sum += relevant_values[i];
            relevant_values[i] = partial_sum.abs() % 10;
        }
    }

    println!(
        "First 8 digits at index {} after 100 FFTs: {}",
        index,
        relevant_values.iter().take(8).map(|x| x.to_string()).collect::<String>()
    );
}

fn main() -> Result<()> {
    let opt = Cli::from_args();

    let f = File::open(opt.file)?;
    let mut reader = BufReader::new(f);
    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;

    let vals = contents.chars().map(|c| {
        c.to_digit(10).map(|x| x as i64).ok_or(From::from("Could not parse character to digit!"))
    }).collect::<Result<Vec<i64>>>()?;
    part1(&vals);
    part2(&vals);
    Ok(())
}