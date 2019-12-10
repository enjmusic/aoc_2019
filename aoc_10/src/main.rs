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
    #[structopt(short = "n")]
    n: usize,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct Asteroid {
    x: i64,
    y: i64,
}

impl Asteroid {
    fn step(&mut self, dir: (i64, i64)) {
        self.x += dir.0;
        self.y += dir.1;
    }

    fn angle_and_distance(&self, other: &Asteroid) -> (f64, f64) {
        let diff = ((other.x - self.x) as f64, (self.y - other.y) as f64);
        let degrees = 90.0 - diff.1.atan2(diff.0).to_degrees();
        (if degrees < 0.0 { 360.0 + degrees } else { degrees }, diff.1.hypot(diff.0))
    }
}

fn get_ordinal(n: usize) -> &'static str {
    match n % 10 {
        1 => "st",
        2 => "nd",
        3 => "rd",
        _ => "th"
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

fn get_nth_asteroid_destroyed(asteroids: &mut HashSet<Asteroid>, station: &Asteroid, n: usize) -> Result<Asteroid> {
    asteroids.remove(station);
    if asteroids.len() < n { return Err(From::from("Not enough asteroids to destroy")); }

    let mut angles_and_distances: Vec<(Asteroid, (f64, f64))> = asteroids.iter()
        .map(|a| (*a, station.angle_and_distance(a))).collect();
    
    angles_and_distances.sort_by(|(_, (a1, d1)), (_, (a2, d2))| {
        if a1 == a2 {
            d1.partial_cmp(d2).unwrap()
        } else {
            a1.partial_cmp(a2).unwrap()
        }
    });

    // Get Vec of Vecs of asteroids grouped by the same
    // angle in pop() order by their distance from station
    let mut grouped_by_angle: Vec<Vec<Asteroid>> = vec![];
    let mut last_seen = -1.0;
    let mut num_angles_processed = 0;
    for (asteroid, (angle, _)) in angles_and_distances.iter().rev() {
        if last_seen < 0.0 || last_seen != *angle {
            grouped_by_angle.push(vec![]);
            num_angles_processed += 1;
            last_seen = *angle;
        }

        grouped_by_angle[num_angles_processed - 1].push(*asteroid);
    }

    // Keep destroying sorted, ordered asteroids in a circle
    let mut num_destroyed = 0;
    loop {
        for asteroids in grouped_by_angle.iter_mut().rev() {
            if let Some(asteroid) = asteroids.pop() {
                num_destroyed += 1;
                if num_destroyed == n { return Ok(asteroid); }
            }
        }
    }
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
    println!(
        "Best location: {}, {} (detected {} asteroids)",
        best_location.0.x, best_location.0.y, best_location.1
    );

    let nth_destroyed = get_nth_asteroid_destroyed(&mut asteroids, &best_location.0, opt.n)?;
    Ok(println!(
        "{}{} asteroid destroyed was at location: {}, {}",
        opt.n, get_ordinal(opt.n), nth_destroyed.x, nth_destroyed.y
    ))
}