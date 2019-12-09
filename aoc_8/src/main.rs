use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use structopt::StructOpt;
use itertools::Itertools;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short = "f", parse(from_os_str))]
    file: PathBuf,
    #[structopt(short = "w")]
    width: usize,
    #[structopt(short = "h")]
    height: usize,
}

struct Layer {
    rows: Vec<Vec<u8>>,
    counts: Vec<u32>,
}

impl Layer {
    fn new(rows: Vec<Vec<u8>>) -> Layer {
        let counts = rows.iter().fold(vec![0; 10], |acc, v| {
            v.iter().fold(acc, |mut acc, v| { acc[*v as usize] += 1; acc })
        });

        Layer {
            rows: rows,
            counts: counts,
        }
    }

    // Overlay a layer on another, discarding any counts present
    fn overlay(&self, other: &Layer) -> Layer {
        Layer {
            rows: self.rows.iter().zip(other.rows.iter()).map(|(upper_row, lower_row)| {
                upper_row.iter().zip(lower_row.iter()).map(|(upper_pixel, lower_pixel)| {
                    *(if *upper_pixel == 2 { lower_pixel } else { upper_pixel })
                }).collect()
            }).collect::<Vec<Vec<u8>>>(),
            counts: vec![],
        }
    }

    fn print(&self) {
        self.rows.iter().for_each(|row| {
            println!("{}", row.iter().map(|pixel| match pixel {
                1 => '\u{2588}',
                _ => ' ',
            }).collect::<String>());
        });
    }
}

fn main() -> Result<()> {
    let opt = Cli::from_args();

    let f = File::open(opt.file)?;
    let mut reader = BufReader::new(f);
    let mut contents = String::new();
    reader.read_to_string(&mut contents).unwrap();

    if contents.len() % (opt.width * opt.height) != 0 {
        return Err(From::from("Image could not be divided cleanly into layers"));
    }

    let rows = contents.chars().chunks(opt.width).into_iter().map(|chunk| {
        chunk.into_iter()
            .map(|digit| {
                digit.to_digit(10)
                .ok_or(From::from(format!("Not a digit: {}", digit))).map(|i| i as u8)
            }).collect::<Result<Vec<u8>>>()
    }).collect::<Result<Vec<Vec<u8>>>>()?;

    let layers: Vec<Layer> = rows.chunks(opt.height).map(|layer_rows| Layer::new(layer_rows.to_vec())).collect();
    let counts_for_least_zeroes = layers.iter().fold((std::u32::MAX, 0, 0), |acc, l| {
        if l.counts[0] < acc.0 { (l.counts[0], l.counts[1], l.counts[2]) } else { acc }
    });

    println!(
        "The layer with the fewest zeroes ({}) has [{} ones] * [{} twos] = {}",
        counts_for_least_zeroes.0,
        counts_for_least_zeroes.1,
        counts_for_least_zeroes.2,
        counts_for_least_zeroes.1 * counts_for_least_zeroes.2
    );

    let blank_layer = Layer::new(vec![vec![2; opt.width]; opt.height]);
    let overlaid_image = layers.iter().fold(blank_layer, |overlay, curr| overlay.overlay(curr));
    overlaid_image.print();

    Ok(())
}
