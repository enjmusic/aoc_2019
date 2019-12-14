use std::fs::File;
use std::cmp;
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use std::collections::HashMap;
use std::time;
use std::thread;
use structopt::StructOpt;
use intcode::program::{Event, IntcodeProgram};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short = "f", parse(from_os_str))]
    file: PathBuf,
    #[structopt(short = "d")]
    display: bool,
}

#[derive(PartialEq)]
enum TileID {
    Empty,
    Wall,
    Block,
    Paddle,
    Ball
}

impl TileID {
    fn from_int(i: i64) -> Result<TileID> {
        match i {
            0 => Ok(TileID::Empty),
            1 => Ok(TileID::Wall),
            2 => Ok(TileID::Block),
            3 => Ok(TileID::Paddle),
            4 => Ok(TileID::Ball),
            _ => Err(From::from(format!("Invalid tile ID: {}", i)))
        }
    }

    fn to_char(&self) -> char {
        match self {
            TileID::Empty => '\u{2002}',
            TileID::Wall => '\u{2592}',
            TileID::Block => '\u{2591}',
            TileID::Paddle => '\u{25AD}',
            TileID::Ball => '\u{2022}',
        }
    }
}

struct Screen {
    tiles: HashMap<(i64, i64), TileID>,
}

impl Screen {
    fn draw(&mut self, x: i64, y: i64, tile_id: i64) -> Result<()> {
        self.tiles.insert((x, y), TileID::from_int(tile_id)?);
        Ok(())
    }

    fn display(&self) {
        let upper_bounds = self.tiles.iter()
            .fold((0, 0), |acc, (c, _)| (cmp::max(acc.0, (*c).0), cmp::max(acc.1, (*c).1)));

        for y in 0..=upper_bounds.1 {
            let line = (0..=upper_bounds.0).map(|x| 
                self.tiles.get(&(x, y)).unwrap_or(&TileID::Empty).to_char()).collect::<String>();
            println!("{}", line);
        }
    }

    fn get_current_move(&self) -> i64 {
        let (mut paddle_x, mut ball_x) = (0, 0);
        self.tiles.iter().for_each(|(c, tid)| { match tid {
            TileID::Paddle => paddle_x = (*c).0,
            TileID::Ball => ball_x = (*c).0,
            _ => (),
        }});
        ball_x.cmp(&paddle_x) as i64
    }
}

fn part1(input: &String) -> Result<()> {
    let mut program = IntcodeProgram::from_raw_input(input)?;
    let mut screen = Screen{ tiles: HashMap::new() };
    program.execute()?;

    for draw_vals in program.get_all_output().chunks(3) {
        if draw_vals.len() != 3 { return Err(From::from("# outputs not divisible by 3")); }
        screen.draw(draw_vals[0], draw_vals[1], draw_vals[2])?;
    }

    Ok(println!("Num block tiles: {}", screen.tiles.iter()
        .fold(0, |acc, (_, v)| acc + if *v == TileID::Block { 1 } else { 0 })))
}

fn part2(input: &String, display: bool) -> Result<()> {
    let mut memory = IntcodeProgram::raw_to_memory(input)?;
    memory[0] = 2; // 2 quarters

    let mut program = IntcodeProgram::from_memory(memory);
    let mut screen = Screen{ tiles: HashMap::new() };
    let mut score = 0;
    let mut outputs = vec![];
    loop {
        match program.execute_until_event()? {
            Event::InputRequired => {
                if display {
                    println!("{}[2J", 27 as char);
                    println!("Score: {}", score);
                    screen.display();
                    thread::sleep(time::Duration::from_millis(125));
                }
                program.give_input(screen.get_current_move());
            },
            Event::ProducedOutput => {
                outputs.push(program.get_output().unwrap());
                if outputs.len() == 3 {
                    match (outputs[0], outputs[1]) {
                        (-1, 0) => score = outputs[2],
                        (x, y) => screen.draw(x, y, outputs[2])?,
                    }
                    outputs.clear();
                }
            },
            Event::Exited => break
        }
    }

    Ok(println!("Final score: {}", score))
}

fn main() -> Result<()> {
    let opt = Cli::from_args();

    let f = File::open(opt.file)?;
    let mut reader = BufReader::new(f);
    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;

    part1(&contents)?;
    part2(&contents, opt.display)
}