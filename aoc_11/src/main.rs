use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;
use std::collections::HashMap;
use structopt::StructOpt;
use intcode::{io, program::IntcodeProgram};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short = "f", parse(from_os_str))]
    file: PathBuf,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct Position {
    x: i64,
    y: i64,
}

#[derive(Clone, Copy)]
enum Direction {
    Up, Down, Left, Right
}

impl Position {
    fn apply_direction(&mut self, dir: Direction) {
        match dir {
            Direction::Up => self.y += 1,
            Direction::Down => self.y -= 1,
            Direction::Left => self.x -= 1,
            Direction::Right => self.x += 1,
        }
    }
}

impl Direction {
    fn turn(&self, right: bool) -> Direction {
        match self {
            Direction::Up => if right { Direction::Right } else { Direction::Left },
            Direction::Down => if right { Direction::Left } else { Direction::Right },
            Direction::Left => if right { Direction::Up } else { Direction::Down },
            Direction::Right => if right { Direction::Down } else { Direction::Up },
        }
    }
}

fn paint_squares(
    paint_state: HashMap<Position, bool>,
    start_pos: Position,
    robot_out: Receiver<i64>,
    robot_in: Sender<i64>
) -> HashMap<Position, bool> {

    let mut out: HashMap<Position, bool> = paint_state.clone();
    let mut curr_position = start_pos;
    let mut curr_direction = Direction::Up;

    loop {
        let send_result = robot_in.send(*out.get(&curr_position).unwrap_or(&false) as i64);
        let color_output = robot_out.recv();
        let turn_output = robot_out.recv();

        if let (Ok(_), Ok(color), Ok(turn)) = (send_result, color_output, turn_output) {
            out.insert(curr_position, color != 0);
            curr_direction = curr_direction.turn(turn != 0);
            curr_position.apply_direction(curr_direction);
        } else {
            break // The robot has finished executing and closed the channels
        }
    }

    out
}

fn run_robot_and_paint(
    program: &String,
    start_square_color: bool,
) -> Result<HashMap<Position, bool>> {

    let mut program = IntcodeProgram::from_raw_input(&program)?;

    let (robot_out_tx, robot_out_rx): (Sender<i64>, Receiver<i64>) = mpsc::channel();
    let (robot_in_tx, robot_in_rx): (Sender<i64>, Receiver<i64>) = mpsc::channel();

    program.replace_input(io::ChannelInputDevice::new(robot_in_rx));
    program.replace_output(io::ChannelOutputDevice::new(robot_out_tx));

    let robot_thread = thread::spawn(move || {
        program.execute().unwrap()
    });

    let mut paint_state = HashMap::new();
    paint_state.insert(Position{ x: 0, y: 0 }, start_square_color);
    let painted_squares = paint_squares(paint_state, Position{ x: 0, y: 0 }, robot_out_rx, robot_in_tx);

    if robot_thread.join().is_err() {
        return Err(From::from("Robot thread panicked"));
    }

    Ok(painted_squares)
}

fn print_painted_squares(squares: HashMap<Position, bool>) {
    let (lower_bounds, upper_bounds) = squares.iter().fold(
        (Position{ x: std::i64::MAX, y: std::i64::MAX }, Position{ x: std::i64::MIN, y: std::i64::MIN }),
        |mut acc, (pos, _)| {
            if pos.x < acc.0.x { acc.0.x = pos.x; }
            if pos.y < acc.0.y { acc.0.y = pos.y; }
            if pos.x > acc.1.x { acc.1.x = pos.x; }
            if pos.y > acc.1.y { acc.1.y = pos.y; }
            acc
        }
    );

    let (width, height) = (1 + upper_bounds.x - lower_bounds.x, 1 + upper_bounds.y - lower_bounds.y);
    for y_base in (0..height).rev() {
        let row_y = y_base + lower_bounds.y;
        println!("{}", (0..width).map(|x_base| {
            let is_white = *squares.get(&Position{ x: x_base + lower_bounds.x, y: row_y }).unwrap_or(&false);
            if is_white { '\u{2588}' } else { ' ' }
        }).collect::<String>());
    }
}

fn main() -> Result<()> {
    let opt = Cli::from_args();

    let f = File::open(opt.file)?;
    let mut reader = BufReader::new(f);
    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;

    // Part 1
    let painted_squares = run_robot_and_paint(&contents, false)?;
    println!("Num squares painted first round: {}", painted_squares.len());

    // Part 2
    let painted_squares = run_robot_and_paint(&contents, true)?;
    Ok(print_painted_squares(painted_squares))
}
