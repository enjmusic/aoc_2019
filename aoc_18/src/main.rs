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

fn key_to_idx(key: char) -> usize { (key as usize) - 97 }

// Get new requirements from existing requirements and unlock mask
fn apply_unlock(mut mask: u32, reqs: &Vec<u8>) -> Vec<u8> {
    let mut out = reqs.clone();
    for i in 0..out.len() { out[i] -= (mask & 1) as u8; mask >>= 1; }
    out
}

// All fields in the key solver are indexed via key_to_idx
struct KeySolver {
    // The distance to each key from the entrance
    entrance_to_keys: Vec<usize>,
    // The distance from each key to each other key
    keys_to_keys: Vec<Vec<usize>>,
    // This contains a bitmask for each key indicating which other keys'
    // requirement counts should be decremented when this one is collected
    unlock_masks: Vec<u32>,
    // This contains a count of how many keys each key is dependent on
    initial_reqs: Vec<u8>,
}

impl KeySolver {
    fn from_grid(grid: &Grid) -> KeySolver {
        let mut keys_sorted = grid.keys.keys().map(|c| *c).collect::<Vec<char>>();
        keys_sorted.sort();

        let mut out = KeySolver{
            entrance_to_keys: std::iter::repeat(0).take(keys_sorted.len()).collect::<Vec<usize>>(),
            keys_to_keys: std::iter::repeat(vec![]).take(keys_sorted.len()).collect::<Vec<Vec<usize>>>(),
            unlock_masks: std::iter::repeat(0).take(keys_sorted.len()).collect::<Vec<u32>>(),
            initial_reqs: std::iter::repeat(0).take(keys_sorted.len()).collect::<Vec<u8>>(),
        };

        let deps_and_distances_from_entrance = grid.bfs_to_keys(grid.entrance, true);
        for (key, dist) in deps_and_distances_from_entrance.distances {
            out.entrance_to_keys[key_to_idx(key)] = dist;
        }

        for (from_key, pos) in grid.keys.iter() {
            let mut distances = std::iter::repeat(0).take(keys_sorted.len()).collect::<Vec<usize>>();
            for (key, dist) in grid.bfs_to_keys(*pos, false).distances {
                distances[key_to_idx(key)] = dist;
            }
            out.keys_to_keys[key_to_idx(*from_key)] = distances;
        }

        for (dependent, deps) in deps_and_distances_from_entrance.dependencies.unwrap() {
            out.initial_reqs[key_to_idx(dependent)] = deps.len() as u8;
            for dep in deps {
                out.unlock_masks[key_to_idx(dep)] |= 1 << key_to_idx(dependent);
            }
        }

        out
    }

    fn get_shortest_path(&mut self) -> usize {
        // A map from (curr_key, keys_acquired) to the min distance seen for that combo thus far
        let mut seen: HashMap<(usize, u32), usize> = HashMap::new();
        // A memoization cache of keys_acquired bitmasks to unlock requirement counts
        let mut cache: HashMap<u32, Vec<u8>> = HashMap::new();
        // The (curr_key, keys_acquired) and current distance for a path
        let mut to_visit: VecDeque<((usize, u32), usize)> = VecDeque::new();

        // Get the bitmask that represents having all keys
        let mut all_keys_mask: u32 = 0;
        for _ in 0..self.unlock_masks.len() { all_keys_mask <<= 1; all_keys_mask |= 1; }

        // Start with the keys that have no requirements
        for idx in 0..self.initial_reqs.len() {
            if self.initial_reqs[idx] == 0 {
                let keys_acquired = 1 << idx;
                let visit_info = ((idx, keys_acquired), self.entrance_to_keys[idx]);
                to_visit.push_front(visit_info);
                seen.insert(visit_info.0, visit_info.1);
                cache.insert(keys_acquired, apply_unlock(self.unlock_masks[idx], &self.initial_reqs));
            }
        }

        let mut best_distance = std::usize::MAX;
        let mut states_visited = 0;
        while to_visit.len() != 0 {
            states_visited += 1;
            let ((curr_key, acquired_mask), curr_dist) = to_visit.pop_back().unwrap();

            if acquired_mask == all_keys_mask {
                if curr_dist < best_distance { best_distance = curr_dist; }
                continue
            }

            let reqs = cache[&acquired_mask].clone();
            for (idx, req_count) in reqs.iter().enumerate() {
                if idx != curr_key && *req_count == 0 && ((acquired_mask >> idx) & 1) == 0 {
                    // This key hasn't been acquired and has no requirements
                    let new_acquired_mask = acquired_mask | (1 << idx);
                    let visit_info = ((idx, new_acquired_mask), curr_dist + self.keys_to_keys[curr_key][idx]);
                    if !seen.contains_key(&visit_info.0) || seen[&visit_info.0] > visit_info.1 {
                        // We haven't seen this (key, acquired) combo or if we have, it was
                        // at a not as optimal distance from the entrance. Visit it!
                        to_visit.push_front(visit_info);
                        seen.insert(visit_info.0, visit_info.1);
                        cache.insert(new_acquired_mask, apply_unlock(self.unlock_masks[idx], &cache[&acquired_mask]));
                    }
                }
            }
        }
        println!("States visited: {}", states_visited);
        best_distance
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
        // (distance from start, point to visit, doors seen so far i.e. dependencies)
        let mut to_visit: VecDeque<(usize, (usize, usize), HashSet<char>)> = VecDeque::new();
        to_visit.push_front((0, start, HashSet::new()));
        let mut visited: HashSet<(usize, usize)> = HashSet::new();
        let mut distances: HashMap<char, usize> = HashMap::new();
        let mut dependencies: HashMap<char, HashSet<char>> = HashMap::new();
        while to_visit.len() != 0 {
            let (mut dist, point, mut deps) = to_visit.pop_back().unwrap();
            visited.insert(point);
            let curr_char = self.data[point.1][point.0];
            if curr_char.is_ascii_uppercase() { deps.insert(curr_char.to_ascii_lowercase()); }
            if curr_char.is_ascii_lowercase() {
                distances.insert(curr_char, dist);
                dependencies.insert(curr_char, deps.clone());
            }
            dist += 1;
            // Calculate next points to search
            for next in vec![(point.0, point.1 - 1), (point.0, point.1 + 1), (point.0 - 1, point.1), (point.0 + 1, point.1)] {
                if self.data[next.1][next.0] != '#' && !visited.contains(&(next.0, next.1)) {
                    to_visit.push_front((dist, (next.0, next.1), deps.clone()));
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