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

// A label location that has not yet been associated with a grid location
enum RawLabel {
    Vertical { x: usize, y1: usize, y2: usize },
    Horizontal { y: usize, x1: usize, x2: usize }
}

fn attempt_label_completion(
    incomplete: &HashMap<(i64, i64), char>,
    letter: char,
    pos: (i64, i64)
) -> Option<(String, RawLabel)> {
    if let Some(c) = incomplete.get(&(pos.0, pos.1 - 1)) {
        Some((
            vec![*c, letter].into_iter().collect(),
            RawLabel::Vertical{ x: pos.0 as usize, y1: (pos.1 - 1) as usize, y2: pos.1 as usize }
        ))
    } else if let Some(c) = incomplete.get(&(pos.0, pos.1 + 1)) {
        Some((
            vec![letter, *c].into_iter().collect(),
            RawLabel::Vertical{ x: pos.0 as usize, y1: pos.1 as usize, y2: (pos.1 + 1) as usize }
        ))
    } else if let Some(c) = incomplete.get(&(pos.0 - 1, pos.1)) {
        Some((
            vec![*c, letter].into_iter().collect(),
            RawLabel::Horizontal{ y: pos.1 as usize, x1: (pos.0 - 1) as usize, x2: pos.0 as usize }
        ))
    } else if let Some(c) = incomplete.get(&(pos.0 + 1, pos.1)) {
        Some((
            vec![letter, *c].into_iter().collect(),
            RawLabel::Horizontal{ y: pos.1 as usize, x1: pos.0 as usize, x2: (pos.0 + 1) as usize }
        ))
    } else {
        None
    }
}

fn get_next(pos: (usize, usize)) -> Vec<(usize, usize)> {
    vec![(pos.0 + 1, pos.1), (pos.0 - 1, pos.1), (pos.0, pos.1 + 1), (pos.0, pos.1 - 1)]
}

fn unvisited_with_min_dist(u: &HashSet<(usize, usize)>, d: &HashMap<(usize, usize), usize>) -> (usize, usize) {
    let (mut min_dist, mut best) = (std::usize::MAX, (0, 0));
    for node in u {
        if d[node] < min_dist { min_dist = d[node]; best = *node; }
    }
    best
}

struct MazeGraph {
    distances: HashMap<(usize, usize), HashMap<(usize, usize), usize>>,
    labels: HashMap<String, Vec<(usize, usize)>>,
}

impl MazeGraph {
    fn from_reader(r: BufReader<File>) -> Result<MazeGraph> {
        let mut raw_labels: HashMap<String, Vec<RawLabel>> = HashMap::new();
        let mut incomplete_labels: HashMap<(i64, i64), char> = HashMap::new();
        let mut grid: Vec<Vec<char>> = vec![];

        for (row, line) in r.lines().enumerate() {
            let mut grid_row = vec![];
            for (col, c) in line?.chars().enumerate() {
                let signed_coords = (col as i64, row as i64);
                match c {
                    'A'..='Z' => {
                        if let Some((label, pos)) = attempt_label_completion(&incomplete_labels, c, signed_coords) {
                            let entry = raw_labels.entry(label).or_insert(vec![]);
                            entry.push(pos);
                        } else {
                            incomplete_labels.insert(signed_coords, c);
                        }
                        grid_row.push(' ');
                    },
                    _ => grid_row.push(c)
                }
            }
            grid.push(grid_row);
        }

        let mut nodes: HashSet<(usize, usize)> = HashSet::new();
        let labels = raw_labels.iter().map(|(s, raws)| {
            (s.clone(), raws.iter().map(|raw| {
                let node = match raw {
                    RawLabel::Vertical{x, y1, y2} => {
                        if *y1 == 0 {
                            (*x, y2 + 1)
                        } else if grid[y1 - 1][*x] == '.' {
                            (*x, y1 - 1)
                        } else {
                            (*x, y2 + 1)
                        }
                    },
                    RawLabel::Horizontal{y, x1, x2} => {
                        if *x1 == 0 {
                            (x2 + 1, *y)
                        } else if grid[*y][x1 - 1] == '.' {
                            (x1 - 1, *y)
                        } else {
                            (x2 + 1, *y)
                        }
                    }
                };
                nodes.insert(node);
                node
            }).collect::<Vec<(usize, usize)>>())
        }).collect::<HashMap<String, Vec<(usize, usize)>>>();

        // Get normal travel distances between portals inside the maze using BFS
        let mut distances: HashMap<(usize, usize), HashMap<(usize, usize), usize>> = HashMap::new();
        for node in &nodes {
            let mut node_distances: HashMap<(usize, usize), usize> = HashMap::new();
            let mut visited: HashSet<(usize, usize)> = HashSet::new();
            let mut to_visit: VecDeque<((usize, usize), usize)> = VecDeque::new();
            visited.insert(*node);
            to_visit.push_front((*node, 0));
            while to_visit.len() != 0 {
                let (curr_pos, curr_dist) = to_visit.pop_back().unwrap();
                if nodes.contains(&curr_pos) && curr_pos != *node { node_distances.insert(curr_pos, curr_dist); }
                for next in get_next(curr_pos) {
                    if !visited.contains(&next) && grid[next.1][next.0] == '.' {
                        visited.insert(next);
                        to_visit.push_front((next, curr_dist + 1));
                    }
                }
            }
            distances.insert(*node, node_distances);
        }

        // Set travel distances between the nodes for a given portal to 1
        for (_, nodes) in &labels {
            if nodes.len() == 2 {
                distances.get_mut(&nodes[0].clone()).map_or(None, |d| d.insert(nodes[1], 1) );
                distances.get_mut(&nodes[1].clone()).map_or(None, |d| d.insert(nodes[0], 1) );
            }
        }

        Ok(MazeGraph{ distances: distances, labels: labels })
    }

    fn get_shortest_path_length(&self, start: String, end: String) -> usize {
        let start_point = self.labels[&start][0];
        let end_point = self.labels[&end][0];
        let mut unvisited: HashSet<(usize, usize)> = self.distances.keys().map(|p| *p).collect();
        let mut distances: HashMap<(usize, usize), usize> = self.distances.keys().map(|p| (*p, std::usize::MAX)).collect();
        distances.insert(start_point, 0);

        // Run Dijkstra's algorithm
        while unvisited.len() != 0 {
            let next_visit = unvisited_with_min_dist(&unvisited, &distances);
            unvisited.remove(&next_visit);
            for (neighbor_pos, dist_to_neighbor) in &self.distances[&next_visit] {
                let alt = distances[&next_visit] + dist_to_neighbor;
                if alt < distances[neighbor_pos] { distances.insert(*neighbor_pos, alt); }
            }
        }

        distances[&end_point]
    }
}

fn main() -> Result<()> {
    let opt = Cli::from_args();

    let f = File::open(opt.file)?;
    let reader = BufReader::new(f);
    let graph = MazeGraph::from_reader(reader)?;
    println!("Shortest from AA to ZZ is {}", graph.get_shortest_path_length("AA".to_owned(), "ZZ".to_owned()));
    Ok(())
}