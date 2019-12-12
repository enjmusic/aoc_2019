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
    position: (i64, i64, i64),
    velocity: (i64, i64, i64)
}

impl Moon {
    fn parse(s: String) -> Result<Moon> {
        lazy_static! {
            static ref MOON_PARSE_REGEX: Regex = 
                Regex::new(r"<x=(-?\d+), y=(-?\d+), z=(-?\d+)>").unwrap();
        }

        if let Some(captures) = MOON_PARSE_REGEX.captures(s.as_str()) {
            Ok(Moon{
                position: (
                    captures[1].parse::<i64>()?,
                    captures[2].parse::<i64>()?,
                    captures[3].parse::<i64>()?
                ),
                velocity: (0, 0, 0),
            })
        } else {
            Err(From::from("Could not parse moon"))
        }
    }

    fn total_energy(&self) -> i64 {
        (self.position.0.abs() + self.position.1.abs() + self.position.2.abs())
        * (self.velocity.0.abs() + self.velocity.1.abs() + self.velocity.2.abs())
    }

    fn get_gravity_towards(&self, other: &Moon) -> (i64, i64, i64) {
        (
            (other.position.0 - self.position.0).signum(),
            (other.position.1 - self.position.1).signum(),
            (other.position.2 - self.position.2).signum()
        )
    }

    fn apply_gravity(&mut self, gravity: (i64, i64, i64)) {
        self.velocity = (
            self.velocity.0 + gravity.0,
            self.velocity.1 + gravity.1,
            self.velocity.2 + gravity.2
        );
    }

    fn apply_velocity(&mut self) {
        self.position = (
            self.position.0 + self.velocity.0,
            self.position.1 + self.velocity.1,
            self.position.2 + self.velocity.2
        );
    }
}

fn simulate_moons(moons: &mut Vec<Moon>) {
    for i in 0..moons.len() {
        for j in i..moons.len() {
            let gravity = moons[i].get_gravity_towards(&moons[j]);
            moons[i].apply_gravity(gravity);
            moons[j].apply_gravity((-gravity.0, -gravity.1, -gravity.2));
        }
        moons[i].apply_velocity();
    }
}

fn moons_to_dimensional_hash_keys(moons: &Vec<Moon>) -> (String, String, String) {
    let mut keys = ("".to_owned(), "".to_owned(), "".to_owned());
    for moon in moons {
        keys.0.push_str(&format!("[{},{}]", moon.position.0, moon.velocity.0));
        keys.1.push_str(&format!("[{},{}]", moon.position.1, moon.velocity.1));
        keys.2.push_str(&format!("[{},{}]", moon.position.2, moon.velocity.2));
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
    let mut seen_x_states: HashMap<String, usize> = HashMap::new();
    let mut seen_y_states: HashMap<String, usize> = HashMap::new();
    let mut seen_z_states: HashMap<String, usize> = HashMap::new();
    let mut x_cycle: Option<usize> = None;
    let mut y_cycle: Option<usize> = None;
    let mut z_cycle: Option<usize> = None;
    let mut num_steps = 0;

    while x_cycle.is_none() || y_cycle.is_none() || z_cycle.is_none() {
        let (x_key, y_key, z_key) = moons_to_dimensional_hash_keys(moons);

        if let Some(v) = seen_x_states.get(&x_key) {
            if x_cycle.is_none() { x_cycle = Some(num_steps - *v); }
        }

        if let Some(v) = seen_y_states.get(&y_key) {
            if y_cycle.is_none() { y_cycle = Some(num_steps - *v); }
        }

        if let Some(v) = seen_z_states.get(&z_key) {
            if z_cycle.is_none() { z_cycle = Some(num_steps - *v); }
        }

        simulate_moons(moons);
        if x_cycle.is_none() { seen_x_states.insert(x_key, num_steps); }
        if y_cycle.is_none() { seen_y_states.insert(y_key, num_steps); }
        if z_cycle.is_none() { seen_z_states.insert(z_key, num_steps); }
        num_steps += 1;
    }

    let lcm_of_dimension_cycles = lcm(lcm(x_cycle.unwrap(), y_cycle.unwrap()), z_cycle.unwrap());
    println!("Would return to previously seen state after {} simulation steps", lcm_of_dimension_cycles);
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
