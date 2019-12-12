#[macro_use] extern crate lazy_static;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use std::collections::HashMap;
use structopt::StructOpt;
use regex::Regex;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(StructOpt)]
struct Cli {
    #[structopt(short = "f", parse(from_os_str))]
    file: PathBuf,
    #[structopt(short = "n")]
    num_steps: usize,
}

#[derive(Clone, Copy, Debug)]
struct Moon {
    position: [i64; 3],
    velocity: [i64; 3],
}

impl Moon {
    fn parse(s: String) -> Result<Moon> {
        lazy_static! {
            static ref MOON_PARSE_REGEX: Regex = 
                Regex::new(r"<x=(-?\d+), y=(-?\d+), z=(-?\d+)>").unwrap();
        }

        if let Some(captures) = MOON_PARSE_REGEX.captures(s.as_str()) {
            Ok(Moon{
                position: [
                    captures[1].parse::<i64>()?,
                    captures[2].parse::<i64>()?,
                    captures[3].parse::<i64>()?
                ],
                velocity: [0, 0, 0],
            })
        } else {
            Err(From::from("Could not parse moon"))
        }
    }

    fn total_energy(&self) -> i64 {
        self.position.iter().map(|p| p.abs()).sum::<i64>()
        * self.velocity.iter().map(|p| p.abs()).sum::<i64>()
    }

    fn get_gravity_towards(&self, other: &Moon) -> [i64; 3] {
        let as_vec: Vec<i64> = other.position.iter().zip(self.position.iter())
            .map(|(o, s)| (o - s).signum()).collect();
        [as_vec[0], as_vec[1], as_vec[2]]
    }

    fn apply_gravity(&mut self, gravity: [i64; 3], neg: bool) {
        self.velocity.iter_mut().zip(gravity.iter()).for_each(|(v, g)| *v += if neg { -g } else { *g });
    }

    fn apply_velocity(&mut self) {
        self.position.iter_mut().zip(self.velocity.iter()).for_each(|(p, v)| *p += v);
    }
}

fn simulate_moons(moons: &mut Vec<Moon>) {
    for i in 0..moons.len() {
        for j in i..moons.len() {
            let gravity = moons[i].get_gravity_towards(&moons[j]);
            moons[i].apply_gravity(gravity, false);
            moons[j].apply_gravity(gravity, true);
        }
        moons[i].apply_velocity();
    }
}

fn moons_to_dimensional_hash_keys(moons: &Vec<Moon>) -> [String; 3] {
    let mut keys = ["".to_owned(), "".to_owned(), "".to_owned()];
    for moon in moons {
        for i in 0..3 {
            keys[i].push_str(&format!("[{},{}]", moon.position[i], moon.velocity[i]));
        }
    }
    keys
}

fn lcm(a: usize, b: usize) -> usize {
    let mut params = if a >= b { (a, b) } else { (b, a) };
    while params.1 != 0 {
        params = (params.1, params.0 % params.1);
    }
    (a * b) / params.0
}

fn part1(moons: &mut Vec<Moon>, num_steps: usize) {
    for _ in 0..num_steps {
        simulate_moons(moons);
    }

    let total_energy = moons.iter().fold(0, |acc, m| m.total_energy() + acc);
    println!("Total energy after {} simulation steps: {}", num_steps, total_energy)
}

fn part2(moons: &mut Vec<Moon>) {
    let mut seen_states: [HashMap<String, usize>; 3] = [HashMap::new(), HashMap::new(), HashMap::new()];
    let mut cycle_lengths: [Option<usize>; 3] = [None, None, None];
    let mut num_steps = 0;

    while !cycle_lengths.iter().all(|l| l.is_some() ) {
        let hash_keys = moons_to_dimensional_hash_keys(moons);
        for i in 0..3 {
            if cycle_lengths[i].is_none() {
                if let Some(v) = seen_states[i].get(&hash_keys[i]) {
                    cycle_lengths[i] = Some(num_steps - *v);
                }
                seen_states[i].insert(hash_keys[i].clone(), num_steps);
            }
        }

        simulate_moons(moons);
        num_steps += 1;
    }

    let lcm_across_dimensions = cycle_lengths.iter().fold(1, |acc, l| lcm(acc, l.unwrap()));
    println!("Would return to previously seen state after {} simulation steps", lcm_across_dimensions);
}

fn main() -> Result<()> {
    let opt = Cli::from_args();

    let f = File::open(opt.file)?;
    let reader = BufReader::new(f);
    let mut moons: Vec<Moon> = vec![];
    for line in reader.lines() {
        moons.push(Moon::parse(line?)?);
    }

    part1(&mut moons.clone(), opt.num_steps);
    part2(&mut moons);
    Ok(())
}
