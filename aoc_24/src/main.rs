use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use std::collections::{HashMap, HashSet};
use structopt::StructOpt;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short = "f", parse(from_os_str))]
    file: PathBuf,
}

struct RecursiveCells {
    layers: HashMap<i32, u32>,
    min: i32,
    max: i32,
}

impl RecursiveCells {
    fn new(start: u32) -> RecursiveCells {
        RecursiveCells{ layers: vec![(0, start)].into_iter().collect(), min: 0, max: 0 }
    }

    fn step(&mut self) {
        if self.layers[&self.min] != 0 {
            self.layers.insert(self.min - 1, 0);
            self.min -= 1;
        }
        if self.layers[&self.max] != 0 {
            self.layers.insert(self.max + 1, 0);
            self.max += 1;
        }
        let mut new_layers = HashMap::new();
        for (&layer, &cells) in &self.layers {
            let cells_stepped = step(cells, layer, &|bit, layer| {
                all_checks(bit, layer).fold(0, |acc, (check_bit, check_layer)| {
                    acc + if let Some(layer_bits) = self.layers.get(&check_layer) {
                        ((layer_bits >> check_bit) & 1)
                    } else {
                        0
                    }
                })
            }) & 0x1ffefff; // Empty out the center cell
            new_layers.insert(layer, cells_stepped);
        }
        self.layers = new_layers;
    }

    fn count_alive(&self) -> usize {
        self.layers.iter().fold(0, |acc, (_, bits)| acc + bits.count_ones() as usize)
    }
}

fn to_cells(input: &String) -> u32 {
    let mut out: u32 = 0;
    for c in input.chars() {
        if !c.is_whitespace() {
            out = (out >> 1) | (((c == '#') as u32) << 24);
        }
    }
    out
}

fn all_checks(bit: i32, layer: i32) -> impl Iterator<Item = (i32, i32)> {
    internal_layer_checks(bit, layer).chain(external_layer_checks(bit, layer)).chain(same_layer_checks(bit, layer, true))
}

fn internal_layer_checks(bit: i32, layer: i32) -> impl Iterator<Item = (i32, i32)> {
    match bit {
        7 => (0..5).collect(),
        11 => (0..5).map(|i| i * 5).collect(),
        13 => (0..5).map(|i| 4 + i * 5).collect(),
        17 => (20..25).collect(),
        _ => vec![],
    }.into_iter().zip(std::iter::repeat(layer + 1))
}

fn external_layer_checks(bit: i32, layer: i32) -> impl Iterator<Item = (i32, i32)> {
    let to_check_horizontal = match bit % 5 {
        0 => vec![11],
        4 => vec![13],
        _ => vec![],
    };
    let to_check_vertical = match bit / 5 {
        0 => vec![7],
        4 => vec![17],
        _ => vec![],
    };
    to_check_horizontal.into_iter().chain(to_check_vertical.into_iter()).zip(std::iter::repeat(layer - 1))
}

fn same_layer_checks(bit: i32, layer: i32, exclude_center: bool) -> impl Iterator<Item = (i32, i32)> {
    let to_check_horizontal = match bit % 5 {
        0 => vec![bit + 1],
        4 => vec![bit - 1],
        _ => vec![bit - 1, bit + 1],
    };
    let to_check_vertical = match bit / 5 {
        0 => vec![bit + 5],
        4 => vec![bit - 5],
        _ => vec![bit - 5, bit + 5],
    };
    to_check_horizontal.into_iter().chain(to_check_vertical.into_iter())
        .filter(move |&b| b != 12 || !exclude_center).zip(std::iter::repeat(layer))
}

fn get_next_state(curr_state: u32, neighbor_alive_counts: u32) -> u32 {
    match (curr_state, neighbor_alive_counts) {
        (0, 1) | (0, 2) | (1, 1) => 1,
        (1, _) => 0,
        (prev, _) => prev,
    }
}

fn step(cells: u32, layer: i32, alive_counts_fn: &dyn Fn(/* bit */ i32,/* layer */ i32) -> u32) -> u32 {
    let mut out: u32 = 0;
    for i in 0..25 {
        out |= get_next_state((cells >> i) & 1, alive_counts_fn(i, layer)) << i;
    }
    out
}

fn part1(mut cells: u32) {
    let mut seen = HashSet::new();
    while !seen.contains(&cells) {
        seen.insert(cells);
        cells = step(cells, 0, &|i, _| {
            same_layer_checks(i, 0, false).fold(0, |acc, (b, _)| ((cells >> b) & 1) + acc)
        });
    }
    println!("Biodiversity rating for first repeated layout: {}", cells);
}

fn part2(cells: u32) {
    let mut rec_cells = RecursiveCells::new(cells);
    for _ in 0..200 { rec_cells.step(); }
    println!("# of bugs after 200 minutes: {}", rec_cells.count_alive());
}

fn main() -> Result<()> {
    let opt = Cli::from_args();

    let f = File::open(opt.file)?;
    let mut reader = BufReader::new(f);
    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;
    
    let cells = to_cells(&contents);
    part1(cells);
    part2(cells);
    Ok(())
}
