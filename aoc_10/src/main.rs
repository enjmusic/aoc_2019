use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use structopt::StructOpt;
use std::collections::HashSet;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short = "f", parse(from_os_str))]
    file: PathBuf,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct Asteroid {
    x: i64,
    y: i64,
}

impl Asteroid {
    fn step(&mut self, dir: (i64, i64)) {
        self.x += dir.0;
        self.y += dir.1;
    }
}

fn gcd(a: i64, b: i64) -> i64 {
    let mut params = if a >= b { (a, b) } else { (b, a) };
    while params.1 != 0 {
        params = (params.1, params.0 % params.1);
    }
    params.0
}

fn get_best_location(asteroids: &HashSet<Asteroid>) -> (Asteroid, i64) {
    asteroids.iter().fold((Asteroid{ x: 0, y: 0}, std::i64::MIN), |res, a1| {
        let num_detected = asteroids.iter().fold(0, |num, a2| {
            if a1 == a2 { return num; }
            let diff = (a2.x - a1.x, a2.y - a1.y);
            let num_steps = gcd(diff.0.abs(), diff.1.abs());
            let step = (diff.0 / num_steps, diff.1 / num_steps);
            let mut a1_step = a1.clone();

            a1_step.step(step);
            while a1_step != *a2 {
                if asteroids.contains(&a1_step) { return num }
                a1_step.step(step);
            }

            num + 1
        });

        if num_detected > res.1 { (*a1, num_detected) } else { res }
    })
}

fn main() -> Result<()> {
    let opt = Cli::from_args();

    let f = File::open(opt.file)?;
    let reader = BufReader::new(f);

    let mut asteroids: HashSet<Asteroid> = HashSet::new();
    for (y, line) in reader.lines().enumerate() {
        for (x, c) in  line?.chars().enumerate() {
            if c == '#' { asteroids.insert(Asteroid{ x: x as i64, y: y as i64 }); }
        }
    }

    let best_location = get_best_location(&asteroids);
    Ok(println!(
        "Best location: {}, {} (detected {} asteroids)",
        best_location.0.x, best_location.0.y, best_location.1
    ))
}