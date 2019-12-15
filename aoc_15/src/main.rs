use std::cmp;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use std::collections::{HashMap, HashSet, VecDeque};
use structopt::StructOpt;
use intcode::program::{Event, IntcodeProgram};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short = "f", parse(from_os_str))]
    file: PathBuf,
}

#[derive(Copy, Clone)]
enum Dir {
    North = 1, South = 2, West = 3, East = 4
}

impl Dir {
    fn to_offset(&self) -> (i64, i64) {
        match self {
            Dir::North => (0, 1),
            Dir::South => (0, -1),
            Dir::East => (1, 0),
            Dir::West => (-1, 0),
        }
    }

    fn reverse(&self) -> Dir {
        match self {
            Dir::North => Dir::South,
            Dir::South => Dir::North,
            Dir::East => Dir::West,
            Dir::West => Dir::East,
        }
    }

    fn get_all() -> Vec<Dir> {
        vec![Dir::North, Dir::South, Dir::East, Dir::West]
    }

    fn apply(&self, dir: (i64, i64)) -> (i64, i64) {
        let offset = self.to_offset();
        (dir.0 + offset.0, dir.1 + offset.1)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Tile {
    Empty, Wall, Oxygen
}

struct Mapper {
    program: IntcodeProgram,
    map: HashMap<(i64, i64), Tile>,
    curr_loc: (i64, i64),
    curr_dir: Dir,
    move_stack: Vec<Dir>,
    option_stack: Vec<Vec<Dir>>,
    reversing: bool,
}

impl Mapper {
    fn init(program_raw: &String) -> Result<Mapper> {
        let mut mapper = Mapper{
            program: IntcodeProgram::from_raw_input(program_raw)?,
            map: HashMap::new(),
            curr_loc: (0, 0),
            curr_dir: Dir::North,
            move_stack: vec![],
            option_stack: vec![Dir::get_all()],
            reversing: false,
        };
        mapper.map.insert((0, 0), Tile::Empty);
        Ok(mapper)
    }

    fn map_section(&mut self) -> Result<()> {
        loop {
            match self.program.execute_until_event()? {
                Event::Exited => return Ok(()),
                Event::InputRequired => {
                    if let Some(dir) = self.option_stack.last_mut().unwrap().pop() {
                        self.reversing = false;
                        self.program.give_input(dir as i64);
                        self.curr_dir = dir;
                    } else if let Some(dir) = self.move_stack.pop() {
                        // Go back to previous square
                        self.option_stack.pop();
                        self.reversing = true;
                        self.curr_dir = dir.reverse();
                        self.program.give_input(self.curr_dir as i64);
                        self.curr_loc = self.curr_dir.apply(self.curr_loc);
                    } else {
                        return Ok(())
                    }
                },
                Event::ProducedOutput => {
                    let output = self.program.get_output().unwrap();
                    if self.reversing { continue }
                    match output {
                        0 => { // Hit a wall
                            self.map.insert(self.curr_dir.apply(self.curr_loc), Tile::Wall);
                        },
                        x => {
                            self.curr_loc = self.curr_dir.apply(self.curr_loc);
                            self.map.insert(self.curr_loc, if x == 1 { Tile::Empty } else { Tile::Oxygen });
                            self.move_stack.push(self.curr_dir);
                            self.option_stack.push(Dir::get_all().iter().filter_map(|x| { // Push unmapped positions
                                if self.map.contains_key(&(x.apply(self.curr_loc))) { None } else { Some(*x) }
                            }).collect());
                        },
                    }
                }
            }
        }
    }

    fn to_grid(&self) -> (Vec<Vec<Tile>>, (i64, i64)) {
        let (mut lower, mut upper) = ((std::i64::MAX, std::i64::MAX), (std::i64::MIN, std::i64::MIN));
        self.map.iter().for_each(|((x, y), _)| {
            lower.0 = cmp::min(*x, lower.0);
            lower.1 = cmp::min(*y, lower.1);
            upper.0 = cmp::max(*x, upper.0);
            upper.1 = cmp::max(*y, upper.1);
        });

        ((lower.1..=upper.1).map(|y| (lower.0..=upper.0).map(|x| {
            *self.map.get(&(x, y)).unwrap_or(&Tile::Empty)
        }).collect::<Vec<Tile>>()).collect::<Vec<Vec<Tile>>>(), (-lower.0, -lower.1))
    }
}

// If a search tile type is provided, return the location of & min distance to a tile of that type.
// If no search tile is provided, return the maximum distance the search had to travel.
fn bfs(map: &Vec<Vec<Tile>>, start: (i64, i64), search: Option<Tile>) -> (Option<(i64, i64)>, usize) {
    let mut max_distance = 0;
    let mut visited: HashSet<(i64, i64)> = HashSet::new();
    let mut to_visit: VecDeque<((i64, i64), usize)> = VecDeque::new();
    to_visit.push_front((start, 0));
    while to_visit.len() != 0 {
        let (visit, dist) = to_visit.pop_back().unwrap();
        if search.is_some() && map[visit.1 as usize][visit.0 as usize] == search.unwrap() {
            return (Some(visit), dist)
        }
        if dist > max_distance { max_distance = dist; }
        visited.insert(visit);
        for dir in Dir::get_all() {
            let offset = dir.to_offset();
            let maybe_next = (visit.0 + offset.0, visit.1 + offset.1);
            if maybe_next.1 < 0 && maybe_next.1 >= map.len() as i64 { continue }
            if maybe_next.0 < 0 && maybe_next.0 >= map[0].len() as i64 { continue }
            if map[maybe_next.1 as usize][maybe_next.0 as usize] != Tile::Wall && !visited.contains(&maybe_next) {
                to_visit.push_front((maybe_next, dist + 1));
            }
        }
    }

    (None, max_distance)
}

fn part1(map: &Vec<Vec<Tile>>, start: (i64, i64)) -> Result<(i64, i64)> {
    if let (Some(oxygen_coords), num_steps) = bfs(map, start, Some(Tile::Oxygen)) {
        println!("Found oxygen at {:?} after {} steps", oxygen_coords, num_steps);
        Ok(oxygen_coords)
    } else {
        Err(From::from("Could not find oxygen"))
    }
}

fn part2(map: &Vec<Vec<Tile>>, oxygen: (i64, i64)) -> Result<()> {
    if let (None, minutes) = bfs(map, oxygen, None) {
        println!("Took {} minutes to fill section with oxygen", minutes);
        Ok(())
    } else {
        Err(From::from("Could not fill section with oxygen"))
    }
}

fn main() -> Result<()> {
    let opt = Cli::from_args();

    let f = File::open(opt.file)?;
    let mut reader = BufReader::new(f);
    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;
    
    let mut mapper = Mapper::init(&contents)?;
    mapper.map_section()?;
    let (map, start) = mapper.to_grid();
    let oxygen_location = part1(&map, start)?;
    part2(&map, oxygen_location)
}
