use std::cmp;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use std::collections::HashMap;
use structopt::StructOpt;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short = "f", parse(from_os_str))]
    file: PathBuf,
}

struct Reaction {
    reagents: Vec<(u64, String)>,
    product: (u64, String),
}

impl Reaction {
    fn from_string(s: String) -> Result<Reaction> {
        let split_on_arrow = s.split(" => ").collect::<Vec<&str>>();
        if split_on_arrow.len() != 2 { return Err(From::from("Invalid reaction")); }
        Ok(Reaction{
            reagents: split_on_arrow[0].split(", ")
                .map(|s| parse_chemical_and_amount(s)).collect::<Result<Vec<(u64, String)>>>()?,
            product: parse_chemical_and_amount(split_on_arrow[1])?
        })
    }
}

fn parse_chemical_and_amount(s: &str) -> Result<(u64, String)> {
    let split_on_whitespace = s.split_whitespace().collect::<Vec<&str>>();
    if split_on_whitespace.len() != 2 { return Err(From::from("Invalid chemical/amount")); }
    Ok((split_on_whitespace[0].parse::<u64>()?, split_on_whitespace[1].to_owned()))
}

fn make_fuel(reactions: &HashMap<String, Reaction>, num_fuel: u64) -> Result<u64> {
    let mut ore_used = 0;
    let mut extra_materials = reactions.iter().map(|(k, _)| (k.clone(), 0)).collect::<HashMap<String, u64>>();
    let mut need = vec![(num_fuel, "FUEL".to_owned())];
    while need.len() != 0 {
        let needed = need.pop().unwrap();
        if needed.1 == "ORE" {
            ore_used += needed.0;
        } else {
            let extra_used = *cmp::min(extra_materials.get(&needed.1).unwrap_or(&0), &needed.0);
            let amt_needed = needed.0 - extra_used;
            extra_materials.get_mut(&needed.1).map_or((), |e| *e -= extra_used);

            let reaction = reactions.get(&needed.1).ok_or::<Box<dyn std::error::Error>>(From::from("Invalid product"))?;
            let reaction_scalar = (amt_needed as f64 / reaction.product.0 as f64).ceil() as u64;
            let num_materials = reaction.product.0 * reaction_scalar;
            if num_materials > amt_needed {
                extra_materials.get_mut(&needed.1).map_or((), |e| *e += num_materials - amt_needed);
            }
            reaction.reagents.iter().for_each(|(amt, name)| need.push((amt * reaction_scalar, name.clone())));
        }
    }

    Ok(ore_used + extra_materials.get(&"FUEL".to_owned()).unwrap_or(&0))
}

fn part1(reactions: &HashMap<String, Reaction>) -> Result<()> {
    Ok(println!("Ore required to make 1 fuel: {}", make_fuel(reactions, 1)?))
}

fn part2(reactions: &HashMap<String, Reaction>) -> Result<()> {
    // Binary search (except factors of 10, which was faster for some reason)
    let (search_factor, desired) = (10, 1000000000000);
    let (mut inc, mut curr_check) = (desired / search_factor, 0);
    loop {
        if make_fuel(reactions, curr_check)? > desired {
            if inc == 1 { break }
            curr_check -= inc;
            inc /= search_factor;
        } else {
            curr_check += inc;
        }
    }
    Ok(println!("Fuel made with 1 trillion ore: {}", curr_check - 1))
}

fn main() -> Result<()> {
    let opt = Cli::from_args();

    let f = File::open(opt.file)?;
    let reader = BufReader::new(f);

    let reactions = reader.lines().map(|l| {
        let reaction = Reaction::from_string(l?)?;
        Ok((reaction.product.1.clone(), reaction))
    }).collect::<Result<HashMap<String, Reaction>>>()?;

    part1(&reactions)?;
    part2(&reactions)
}
