use std::fs::File;
use std::io::{prelude::*, BufReader};
//use std::iter::repeat;
use std::path::PathBuf;
use structopt::StructOpt;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short = "f", parse(from_os_str))]
    file: PathBuf,
}

fn lcm(a: usize, b: usize) -> usize {
    let mut params = if a >= b { (a, b) } else { (b, a) };
    while params.1 != 0 {
        params = (params.1, params.0 % params.1);
    }
    (a * b) / params.0
}

fn apply_fft(vals: &Vec<i64>, original_size: usize) -> Vec<i64> {
    let mut out = vec![0; vals.len()];
    let num_cycles = vals.len() / original_size;

    for i in 0..vals.len() {
        let cycles_before_repeat = lcm((i + 1) * 4, original_size) / original_size;
        out[i] = if cycles_before_repeat >= num_cycles {
            let mut sum = 0;
            for j in 0..vals.len() {
                let phase = ((j + 1) / (i + 1)) & 3; // 0 -> 0, 1 -> 1, 2 -> -1, 3 -> 0
                if phase & 1 != 0 { sum += if phase == 1 { vals[j] } else { -vals[j] }; }
            }
            sum
        } else {
            let full_cycles = num_cycles / cycles_before_repeat;
            let cycles_leftover = num_cycles - full_cycles * cycles_before_repeat;
            let cycles_leftover_idx = original_size * cycles_leftover;

            let (mut full_cycle_sum, mut leftover_cycles_sum) = (0, 0);
            for j in 0..(original_size * cycles_before_repeat) {
                if j == cycles_leftover_idx { leftover_cycles_sum = full_cycle_sum; }
                let phase = ((j + 1) / (i + 1)) & 3; // 0 -> 0, 1 -> 1, 2 -> -1, 3 -> 0
                if phase & 1 != 0 { full_cycle_sum += if phase == 1 { vals[j] } else { -vals[j] }; }
            }

            full_cycle_sum * (full_cycles as i64) + leftover_cycles_sum
        }.abs() % 10;
    }

    out
}

fn part1(vals: &Vec<i64>) {
    let mut input = vals.clone();
    for _ in 0..100 {
        input = apply_fft(&input, input.len());
    }
    println!(
        "First 8 digits after 100 FFTs: {}",
        input.iter().take(8).map(|x| x.to_string()).collect::<String>()
    );
}

fn part2(vals: &Vec<i64>) {
    let original_size = vals.len();
    let mut input = vals.iter().cycle().take(original_size * 1/* 10000 */).map(|x| *x).collect::<Vec<i64>>();
    for _ in 0..100 {
        input = apply_fft(&input, original_size);
    }
    println!(
        "First 8 (TODO: CORRECT INDEX) digits after 100 FFTs: {}",
        input.iter().take(8).map(|x| x.to_string()).collect::<String>()
    );
}

fn main() -> Result<()> {
    let opt = Cli::from_args();

    let f = File::open(opt.file)?;
    let mut reader = BufReader::new(f);
    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;

    let vals = contents.chars().map(|c| c.to_digit(10).map(|x| x as i64).ok_or(From::from("a"))).collect::<Result<Vec<i64>>>()?;
    part1(&vals);
    part2(&vals);
    Ok(())
}