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

#[derive(Debug)]
enum Command {
    Reverse, // A.K.A. deal into new stack
    Cut(i128),
    DealIncrement(i128),
}

impl Command {
    fn from_string(s: &String) -> Result<Command> {
        let split: Vec<&str> = s.split_whitespace().collect();
        match split.len() {
            2 => match (split[0], split[1].parse::<i128>()) {
                ("cut", Ok(n)) => Ok(Command::Cut(n)),
                _ => Err(From::from("Invalid command")),
            },
            4 => match (split[0], split[1], split[2], split[3]) {
                ("deal", "into", "new", "stack") => Ok(Command::Reverse),
                ("deal", "with", "increment", n) => match n.parse::<i128>() {
                    Ok(i) => Ok(Command::DealIncrement(i)),
                    _ => Err(From::from("Invalid command"))
                },
                _ => Err(From::from("Invalid command"))
            },
            _ => Err(From::from("Invalid command"))
        }
    }
}

fn read_commands(reader: BufReader<File>) -> Result<Vec<Command>> {
    reader.lines().map(|line| Command::from_string(&line?)).collect()
}

// Turn all the commands into just one equation of the form ax + b ≡ y (mod deck_size)
fn collapse_to_linear_congruence(commands: &Vec<Command>, deck_size: i128) -> (i128, i128) {
    let mut factor: i128 = 1;
    let mut constant: i128 = 0;
    for command in commands {
        match command {
            Command::Reverse => {
                factor = -factor;
                constant = deck_size - (1 + constant);
            },
            Command::Cut(idx) => {
                constant += if *idx >= 0 { deck_size - idx.abs() } else { idx.abs() };
            },
            Command::DealIncrement(inc) => {
                factor *= inc;
                constant *= inc;
            },
        }
        factor %= deck_size;
        constant %= deck_size;
    }
    (factor, constant)
}

// Returns gcd, x, y such that ax + by = gcd
fn extended_gcd(a: i128, b: i128) -> (i128, i128, i128) {
	if a == 0 { return (b, 0, 1); }
	let (gcd, x, y) = extended_gcd(b % a, a);
    (gcd, (y - (b/a) * x), x)
}

fn fast_modular_exponentiation(base: i128, exponent: i128, modulus: i128) -> i128 {
    let mut powers_of_two = vec![base % modulus];
    let mut curr_power = 2;
    while curr_power < exponent {
        let last_power = *powers_of_two.last().unwrap();
        powers_of_two.push((last_power * last_power) % modulus);
        curr_power <<= 1;
    }
    let mut out = 1;
    let (mut exponent_process, mut power_idx) = (exponent, 0);
    while exponent_process != 0 {
        if (exponent_process & 1) == 1 { out = (out * powers_of_two[power_idx]) % modulus; }
        exponent_process >>= 1;
        power_idx += 1;
    }
    out
}

fn part1(commands: &Vec<Command>) {
    let (start, deck_size) = (2019, 10007);
    let (factor, constant) = collapse_to_linear_congruence(commands, deck_size);
    println!("Card 2019 ended at index: {}", (start * factor + constant).rem_euclid(deck_size))
}

fn part2(commands: &Vec<Command>) {
    let (end, deck_size, num_iters) = (2020, 119315717514047i128, 101741582076661i128);
    let (factor, constant) = collapse_to_linear_congruence(commands, deck_size);

    // First we need to compose the linear congruence num_iters times. It ends up being
    // two terms - (x * factor^num_iters) and the first num_iters terms of the divergent
    // geometric series constant(1 + factor + factor^2 + ... + factor^(num_iters - 1)).
    let factor_exponent = fast_modular_exponentiation(factor, num_iters, deck_size);
    let (_, denominator_inverse, _) = extended_gcd(factor - 1, deck_size);
    let series_sum = (constant * (((factor_exponent - 1) * denominator_inverse) % deck_size)) % deck_size;
    let (new_factor, new_constant) = (factor_exponent, series_sum);

    // 2020 - new_constant ≡ new_factor * x (mod deck_size)
    let lhs = (end - new_constant).rem_euclid(deck_size);

    // We need to get the modular multiplicative inverse of new_factor
    // so we can multiply it to both sides of the equation
    let (_, new_factor_inverse, _) = extended_gcd(new_factor, deck_size);
    let x = (lhs * new_factor_inverse).rem_euclid(deck_size);
    println!("The card at index 2020 started at index {}", x);
}

fn main() -> Result<()> {
    let opt = Cli::from_args();

    let f = File::open(opt.file)?;
    let reader = BufReader::new(f);
    let commands = read_commands(reader)?;
    part1(&commands);
    part2(&commands);
    Ok(())
}
