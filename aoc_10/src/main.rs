use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use structopt::StructOpt;
use std::collections::HashMap;

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

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct Dir {
    x: i64,
    y: i64,
}

impl Asteroid {
    fn dir_and_multiple_between(&self, other: &Asteroid) -> (Dir, i64) {
        let diff = (other.x - self.x, other.y - self.y);
        let multiple = gcd(diff.0.abs(), diff.1.abs());
        (Dir{ x: diff.0 / multiple, y: diff.1 / multiple}, multiple)
    }
}

impl Dir {
    // 0 - 360° clockwise from upwards
    fn angle(&self) -> f64 {
        let degrees = 90.0 - (-self.y as f64).atan2(self.x as f64).to_degrees();
        if degrees < 0.0 { 360.0 + degrees } else { degrees }
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

fn get_viewable_groups_for_all(asteroids: &Vec<Asteroid>) -> HashMap<&Asteroid, HashMap<Dir, Vec<i64>>> {
    let mut ret: HashMap<&Asteroid, HashMap<Dir, Vec<i64>>> = asteroids.iter()
        .map(|a| (a, HashMap::new())).collect();

    for a1 in asteroids {
        for a2 in asteroids {
            if a1 == a2 { continue; }
            let (dir, multiple) = a1.dir_and_multiple_between(a2);
            let multiples_for_dir_for_a1 = ret.get_mut(a1).unwrap().entry(dir).or_insert(vec![]);
            multiples_for_dir_for_a1.push(multiple);
        }
    }

    ret
}

fn get_nth_asteroid_destroyed(station: Asteroid, viewable_groups: &HashMap<Dir, Vec<i64>>, n: usize) -> Result<Asteroid> {
    if viewable_groups.len() <= n { return Err(From::from("Not enough asteroids to destroy")); }

    // Sort directions by angle, sort multiples so nearest ones pop earlier
    let mut with_sorted_multiples = viewable_groups.iter()
        .map(|(dir, multiples)| {
            let mut multiples_copy = multiples.clone();
            multiples_copy.sort_by(|x, y| y.cmp(x));
            (*dir, multiples_copy)
        }).collect::<Vec<(Dir, Vec<i64>)>>();
    
    with_sorted_multiples.sort_by(|(a1, _), (a2, _)| a1.angle().partial_cmp(&a2.angle()).unwrap());

    // Keep destroying sorted, ordered asteroids in a circle
    let mut num_destroyed = 0;
    loop {
        for asteroids_in_dir in &mut with_sorted_multiples {
            if let (dir, Some(multiple)) = (asteroids_in_dir.0, (*asteroids_in_dir).1.pop()) {
                num_destroyed += 1;
                if num_destroyed == n { 
                    return Ok(Asteroid{
                        x: station.x + dir.x * multiple,
                        y: station.y + dir.y * multiple,
                    });
                }
            }
        }
    }
}

fn main() -> Result<()> {
    let opt = Cli::from_args();

    let f = File::open(opt.file)?;
    let reader = BufReader::new(f);

    let mut asteroids = vec![];
    for (y, line) in reader.lines().enumerate() {
        for (x, c) in  line?.chars().enumerate() {
            if c == '#' { asteroids.push(Asteroid{ x: x as i64, y: y as i64 }); }
        }
    }

    let viewable_groups_for_all = get_viewable_groups_for_all(&asteroids);

    // Part 1
    let best_location = viewable_groups_for_all.iter().fold((Asteroid{x: 0, y: 0}, 0), |acc, (asteroid, groups)| {
        if groups.len() > acc.1 { (**asteroid, groups.len()) } else { acc }
    });
    println!(
        "Best location: {}, {} (detected {} asteroids)",
        best_location.0.x, best_location.0.y, best_location.1
    );

    // Part 2
    let nth_destroyed = get_nth_asteroid_destroyed(best_location.0, &viewable_groups_for_all[&best_location.0], opt.n)?;
    Ok(println!(
        "{}{} asteroid destroyed was at location: {}, {}",
        opt.n, get_ordinal(opt.n), nth_destroyed.x, nth_destroyed.y
    ))
}