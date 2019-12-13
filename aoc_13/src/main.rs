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

    fn get_paddle_and_ball_locations(&self) -> Result<((i64, i64), (i64, i64))> {
        let mut paddle_location: Option<(i64, i64)> = None;
        let mut ball_location: Option<(i64, i64)> = None;

        self.tiles.iter().for_each(|(c, tid)| { match tid {
            TileID::Paddle => paddle_location = Some(*c),
            TileID::Ball => ball_location = Some(*c),
            _ => (),
        }});

        if let (Some(p), Some(b)) = (paddle_location, ball_location) {
            Ok((p, b))
        } else {
            Err(From::from("Could not find both paddle and ball"))
        }
    }
}

struct Predictor {
    ball_position: (i64, i64),
    ball_velocity: (i64, i64),
    paddle_position: (i64, i64),
    target_x: i64,
}

impl Predictor {
    fn new() -> Predictor {
        Predictor{
            ball_position: (0, 0),
            ball_velocity: (0, 0),
            paddle_position: (0, 0),
            target_x: 0,
        }
    }

    fn update(&mut self, screen: &Screen) -> Result<()> {
        let (p, b) = screen.get_paddle_and_ball_locations()?;
        self.ball_position = b;
        self.paddle_position = p;
        self.ball_velocity = (b.0 - self.ball_position.0, b.1 - self.ball_position.1);
        self.target_x = self.ball_position.0 + self.ball_velocity.0;
        Ok(())
    }

    fn get_input(&self) -> i64 {
        self.target_x.cmp(&self.paddle_position.0) as i64
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
    let mut predictor = Predictor::new();
    let mut outputs = vec![];
    loop {
        match program.execute_until_event()? {
            Event::InputRequired => {
                predictor.update(&screen).unwrap_or(());

                if display {
                    println!("{}[2J", 27 as char);
                    println!("Score: {}", score);
                    screen.display();
                    thread::sleep(time::Duration::from_millis(125));
                }

                program.give_input(predictor.get_input());
            },
            Event::ProducedOutput => {
                outputs.push(program.get_output().unwrap());
                if outputs.len() == 3 {
                    if outputs[0] == -1 && outputs[1] == 0 {
                        score = outputs[2];
                    } else {
                        screen.draw(outputs[0], outputs[1], outputs[2])?;
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