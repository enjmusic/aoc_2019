use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use structopt::StructOpt;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short = "f", parse(from_os_str))]
    file: PathBuf,
}

struct Object {
    name: String,
    orbiting: Option<String>,
    orbited_by: Vec<String>,
}

impl Object {
    fn new(name: String) -> Object {
        Object{
            name: name,
            orbiting: None,
            orbited_by: vec![],
        }
    }
}

fn main() -> Result<()> {
    let opt = Cli::from_args();

    let f = File::open(opt.file)?;
    let reader = BufReader::new(f);

    // Get map of objects with their relevant information
    let mut objects: HashMap<String, Object> = HashMap::new();
    for line in reader.lines() {
        let orbit_line = line?;
        let parts: Vec<&str> = orbit_line.split(")").collect();
        if parts.len() != 2 {
            return Err(From::from(format!("Invalid orbit entry: {}", orbit_line)))
        }

        let object_entry = objects.entry(parts[0].to_owned()).or_insert(Object::new(parts[0].to_owned()));
        object_entry.orbited_by.push(parts[1].to_owned());

        let orbiter_entry = objects.entry(parts[1].to_owned()).or_insert(Object::new(parts[1].to_owned()));
        orbiter_entry.orbiting = Some(parts[0].to_owned());
    }

    // Part 1: Do BFS from `COM` to calculate direct/indirect orbit counts
    let mut to_process = VecDeque::from(vec!["COM".to_owned()]);
    let mut orbit_counts: HashMap<String, (u64, u64)> = objects.iter().map(|(k, _)| {
        (k.clone(), (0, 0))
    }).collect();

    while to_process.len() != 0 {
        let object_name = to_process.pop_back().unwrap();
        if let Some(object) = objects.get(&object_name) {
            for orbiter in &object.orbited_by {
                // Need to scope this mutable borrow of orbit_counts so
                // we can borrow it again to update the orbiter counts
                let object_counts = {
                    let counts = orbit_counts.entry(object.name.clone()).or_insert((0, 0));
                    (counts.0, counts.1)
                };

                let orbiter_counts = orbit_counts.entry(orbiter.to_owned()).or_insert((0, 0));
                *orbiter_counts = (1, object_counts.0 + object_counts.1);

                to_process.push_front(orbiter.clone());
            }
        }
    }

    let total_counts = orbit_counts.values().fold((0, 0), |acc, x| (acc.0 + x.0, acc.1 + x.1));
    println!(
        "Counted {} direct orbits and {} indirect orbits (total {})!",
        total_counts.0,
        total_counts.1,
        total_counts.0 + total_counts.1,
    );

    // Part 2 - Do BFS from the object `YOU` are orbiting to calculate distance
    // to the object `SAN` is orbiting. Each entry to process in our queue now
    // includes a distance traveled thus far.
    let you_are_orbiting = objects.get("YOU").map_or(None, |o| o.orbiting.clone())
        .ok_or::<Box<dyn std::error::Error>>(From::from("YOU are not orbiting any objects"))?;

    let santa_is_orbiting = objects.get("SAN").map_or(None, |o| o.orbiting.clone())
        .ok_or::<Box<dyn std::error::Error>>(From::from("SAN is not orbiting any objects"))?;

    let mut to_process = VecDeque::from(vec![(you_are_orbiting, 0)]);
    // Because we can add both orbited and orbiting objects we need a visited list
    let mut visited: HashSet<String> = HashSet::new();
    let mut distance: Option<u64> = None;
    
    while to_process.len() != 0 {
        let (object_name, dist) = to_process.pop_back().unwrap();
        if object_name == santa_is_orbiting {
            distance = Some(dist);
            break
        }

        visited.insert(object_name.clone());

        if let Some(object) = objects.get(&object_name) {
            if let Some(orbited) = &object.orbiting {
                if !visited.contains(orbited) {
                    to_process.push_front((orbited.clone(), dist + 1));
                }
            }

            for orbiter in &object.orbited_by {
                if !visited.contains(orbiter) {
                    to_process.push_front((orbiter.clone(), dist + 1));
                }
            }
        }
    }

    if let Some(d) = distance {
        println!("Minimum orbital transfers from `YOU` to `SAN` is {}", d);
    } else {
        println!("Could not find path from `YOU` to `SAN`");
    }

    Ok(())
}
