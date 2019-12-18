use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use std::collections::{HashMap, HashSet, VecDeque};
use structopt::StructOpt;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short = "f", parse(from_os_str))]
    file: PathBuf,
}

struct TopologicalOrderingIterator {
    dependencies: HashMap<char, HashSet<char>>,
    in_degree: HashMap<char, usize>,
    done: bool,
}

impl TopologicalOrderingIterator {
    fn from_dependencies(deps: HashMap<char, HashSet<char>>) -> TopologicalOrderingIterator {
        let mut in_degree = HashMap::new();
        for (_, depended_on) in &deps {
            for d in depended_on {
                let entry = in_degree.entry(*d).or_insert(0);
                *entry += 1;
            }
        }
        TopologicalOrderingIterator{
            dependencies: deps,
            in_degree: in_degree,
            done: false,
        }
    }
}

impl Iterator for TopologicalOrderingIterator {
    type Item = Vec<char>;

    fn next(&mut self) -> Option<Vec<char>> {
        None
    }
}

struct KeySolver {
    entrance_to_keys: HashMap<char, usize>,
    keys_to_keys: HashMap<char, HashMap<char, usize>>,
    dependencies: HashMap<char, HashSet<char>>,
}

impl KeySolver {
    fn from_grid(grid: &Grid) -> KeySolver {
        let deps_and_distances_from_entrance = grid.bfs_to_keys(grid.entrance, true);
        let mut keys_to_keys = HashMap::new();
        for (name, pos) in grid.keys.iter() {
            keys_to_keys.insert(*name, grid.bfs_to_keys(*pos, false).distances);
        }
        let dependencies = deps_and_distances_from_entrance.dependencies.unwrap();
        KeySolver {
            entrance_to_keys: deps_and_distances_from_entrance.distances,
            keys_to_keys: keys_to_keys,
            dependencies: dependencies,
        }
    }

    fn get_shortest_path(&mut self) -> usize {
        let mut shortest = std::usize::MAX;
        for path in TopologicalOrderingIterator::from_dependencies(self.dependencies.clone()) {
            let mut length = self.entrance_to_keys[&path[0]];
            for i in 1..path.len() {
                length += self.keys_to_keys[&path[i - 1]][&path[i]];
            }
            if length < shortest { shortest = length; }
        }
        shortest
    }
}

struct BFSResult {
    distances: HashMap<char, usize>,
    dependencies: Option<HashMap<char, HashSet<char>>>,
}

struct Grid {
    data: Vec<Vec<char>>,
    keys: HashMap<char, (usize, usize)>,
    entrance: (usize, usize),
}

impl Grid {
    fn from_reader(r: &mut BufReader<File>) -> Result<Grid> {
        let mut data = vec![];
        let mut entrance = (0, 0);
        let mut keys: HashMap<char, (usize, usize)> = HashMap::new();
        for (row, line) in r.lines().enumerate() {
            data.push(line?.chars().collect::<Vec<char>>());
            for (col, c) in data.last().unwrap().iter().enumerate() {
                if c.is_ascii_lowercase() { keys.insert(*c, (col, row)); }
                if *c == '@' { entrance = (col, row); }
            }
        }
        Ok(Grid{
            data: data,
            keys: keys,
            entrance: entrance,
        })
    }

    fn bfs_to_keys(&self, start: (usize, usize), get_deps: bool) -> BFSResult {
        let mut visited: HashSet<(usize, usize)> = HashSet::new();
        // (distance from start, point to visit, doors seen so far i.e. dependencies)
        let mut frontier: VecDeque<(usize, (usize, usize), HashSet<char>)> = VecDeque::new();
        frontier.push_front((0, start, HashSet::new()));

        let mut distances: HashMap<char, usize> = HashMap::new();
        let mut dependencies: HashMap<char, HashSet<char>> = HashMap::new();

        while frontier.len() != 0 {
            let (mut dist, point, mut deps) = frontier.pop_back().unwrap();
            visited.insert(point);
            let curr_char = self.data[point.1][point.0];
            if curr_char.is_ascii_uppercase() { deps.insert(curr_char); }
            if curr_char.is_ascii_lowercase() {
                distances.insert(curr_char, dist);
                dependencies.insert(curr_char, deps.clone());
            }
            dist += 1;
            // Calculate next points to search
            for new_row in vec![point.1 - 1, point.1 + 1] {
                if self.data[new_row][point.0] != '#' && !visited.contains(&(point.0, new_row)) {
                    frontier.push_front((dist, (point.0, new_row), deps.clone()));
                }
            }
            for new_col in vec![point.0 - 1, point.0 + 1] {
                if self.data[point.1][new_col] != '#' && !visited.contains(&(new_col, point.1)) {
                    frontier.push_front((dist, (new_col, point.1), deps.clone()));
                }
            }
        }

        BFSResult {
            distances: distances,
            dependencies: if get_deps { Some(dependencies) } else { None },
        }
    }
}

fn main() -> Result<()> {
    let opt = Cli::from_args();

    let f = File::open(opt.file)?;
    let mut reader = BufReader::new(f);
    let grid = Grid::from_reader(&mut reader)?;
    let mut solver = KeySolver::from_grid(&grid);
    println!("Minimum distance to get all keys: {}", solver.get_shortest_path());

    Ok(())
}