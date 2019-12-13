use std::fs::File;
use std::cmp;
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;
use std::time;
use structopt::StructOpt;
use intcode::{io, program::IntcodeProgram};

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
        let upper_bounds = self.tiles.iter().fold((0, 0), |acc, (c, _)| {
            (cmp::max(acc.0, (*c).0), cmp::max(acc.1, (*c).1))
        });

        for y in 0..upper_bounds.1 {
            let line = (0..upper_bounds.0).map(|x| self.tiles.get(&(x, y))
                .map_or('\u{2002}', |tid| tid.to_char())).collect::<String>();
            println!("{}", line);
        }
    }

    fn get_paddle_and_ball_locations(&self) -> Result<((i64, i64), (i64, i64))> {
        let mut paddle_location: Option<(i64, i64)> = None;
        let mut ball_location: Option<(i64, i64)> = None;

        self.tiles.iter().for_each(|(c, tid)| {
            match tid {
                TileID::Paddle => paddle_location = Some(*c),
                TileID::Ball => ball_location = Some(*c),
                _ => (),
            }
        });

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

        if self.ball_velocity.1 > 0 {
            let steps_until_hit = (self.paddle_position.1 - 1) - self.ball_position.1;
            self.target_x = self.ball_position.0 + (self.ball_velocity.0 * steps_until_hit);
        } else {
            self.target_x = self.ball_position.0 + self.ball_velocity.0;
        }

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

    let n_block_tiles = screen.tiles.iter().fold(0, |acc, (_, v)| acc + if *v == TileID::Block { 1 } else { 0 });
    println!("Num block tiles: {}", n_block_tiles);

    Ok(())
}

fn part2(input: &String, display: bool) -> Result<()> {
    let mut memory = IntcodeProgram::raw_to_memory(input)?;
    memory[0] = 2; // 2 quarters
    let mut program = IntcodeProgram::from_memory(memory);
    let mut screen = Screen{ tiles: HashMap::new() };

    let (out_tx, out_rx): (Sender<i64>, Receiver<i64>) = mpsc::channel();
    let (in_tx, in_rx): (Sender<i64>, Receiver<i64>) = mpsc::channel();
    let (notify_tx, notify_rx): (Sender<bool>, Receiver<bool>) = mpsc::channel();

    program.replace_output(io::ChannelOutputDevice::new(out_tx));
    program.replace_input(io::NotifyingChannelInputDevice::new(in_rx, notify_tx));

    let program_thread = thread::spawn(move || {
        program.execute().unwrap()
    });

    let mut score = 0;
    let mut predictor = Predictor::new();
    let mut curr_output: [i64; 3] = [0, 0, 0];
    let mut num_outputs_gotten = 0;
    loop {
        // See if it's time to supply input
        match notify_rx.try_recv() {
            Ok(_) => {
                predictor.update(&screen).unwrap_or(());

                if display {
                    println!("{}[2J", 27 as char);
                    screen.display();
                    thread::sleep(time::Duration::from_millis(125));
                }

                in_tx.send(predictor.get_input()).unwrap_or(());
            },
            Err(mpsc::TryRecvError::Disconnected) => break,
            _ => ()
        }

        // Try to collect or process output
        if num_outputs_gotten == 3 {
            if curr_output[0] == -1 && curr_output[1] == 0 {
                score = curr_output[2];
            } else {
                screen.draw(curr_output[0], curr_output[1], curr_output[2])?;
            }
            num_outputs_gotten = 0;
        } else if let Ok(out) = out_rx.try_recv() {
            curr_output[num_outputs_gotten] = out;
            num_outputs_gotten += 1;
        }
    }

    if program_thread.join().is_err() {
        return Err(From::from("Program thread panicked"));
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