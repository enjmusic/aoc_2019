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

fn open_with_min_f(open: &HashMap<((usize, usize), usize), (usize, usize)>) -> (((usize, usize), usize), (usize, usize)) {
    let (mut min_f, mut best) = (std::usize::MAX, (((0, 0), 0), (0, 0)));
    for (node, (g, h)) in open {
        if g + h < min_f { min_f = g + h; best = (*node, (*g, *h)); }
    }
    best
}

struct MazeGraph {
    distances: HashMap<(usize, usize), HashMap<(usize, usize), usize>>,
    internal_nodes: HashMap<(usize, usize), (usize, usize)>,
    external_nodes: HashMap<(usize, usize), (usize, usize)>,
    labels: HashMap<String, Vec<(usize, usize)>>,
}

impl MazeGraph {
    fn from_reader(r: BufReader<File>) -> Result<MazeGraph> {
        let mut raw_labels: HashMap<String, Vec<RawLabel>> = HashMap::new();
        let mut incomplete_labels: HashMap<(i64, i64), char> = HashMap::new();
        let mut grid: Vec<Vec<char>> = vec![];

        // A long and arduous process to parse characters to labels/positions
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

        // Make internal <-> external node mappings
        let mut internal_nodes: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
        let mut external_nodes: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
        for (_, nodes) in &labels {
            if nodes.len() == 2 {
                let (node0, node1) = (nodes[0].clone(), nodes[1].clone());
                let (internal, external) = 
                if node0.0 == 2 || node0.1 == 2 || node0.0 + 3 == grid[0].len() || node0.1 + 3 == grid.len() {
                    (node1, node0)
                } else {
                    (node0, node1)
                };
                internal_nodes.insert(internal, external);
                external_nodes.insert(external, internal);
            }
        }

        Ok(MazeGraph{
            distances: distances,
            internal_nodes: internal_nodes,
            external_nodes: external_nodes,
            labels: labels,
        })
    }

    fn get_shortest_path_length(&self, start: &String, end: &String, recursive: bool) -> Result<usize> {
        let start_point = (self.labels[start][0], 0);
        let end_point = (self.labels[end][0], 0);
        // Open list contains mapping from (pos, layer) to (g, h)
        let mut open: HashMap<((usize, usize), usize), (usize, usize)> = HashMap::new();
        let mut closed: HashMap<((usize, usize), usize), (usize, usize)> = HashMap::new();
        open.insert(start_point, (0, 0));

        // Run A* Search
        while open.len() != 0 {
            let ((q_pos, q_layer), (q_g, q_h)) = open_with_min_f(&open);
            open.remove(&(q_pos, q_layer));
            for (neighbor_pos, dist_to_neighbor) in &self.distances[&q_pos] {
                if (*neighbor_pos, q_layer) == end_point { return Ok(q_g + dist_to_neighbor); }

                // Account for various teleportation conditions changing layers
                let (pos, mut layer, dist) = 
                if recursive && q_layer == 0 && self.external_nodes.contains_key(neighbor_pos) {
                    (*neighbor_pos, q_layer, *dist_to_neighbor)
                } else if let Some(dest) = self.external_nodes.get(neighbor_pos) {
                    (*dest, q_layer - 1, *dist_to_neighbor + 1)
                } else if let Some(dest) = self.internal_nodes.get(neighbor_pos) {
                    (*dest, q_layer + 1, *dist_to_neighbor + 1)
                } else {
                    (*neighbor_pos, q_layer, *dist_to_neighbor)
                };
                if !recursive { layer = 0; } // Ignore layer changes in the non-recursive case

                // Heuristic for distance to end is just layer - very close to Dijkstra's
                let (g, h) = (q_g + dist, q_layer);

                if let Some((open_g, open_h)) = open.get(&(pos, layer)) {
                    if open_g + open_h < g + h { continue }
                }
                if let Some((closed_g, closed_h)) = closed.get(&(pos, layer)) {
                    if closed_g + closed_h < g + h { continue }
                }

                open.insert((pos, layer), (g, h));
                closed.insert((q_pos, q_layer), (q_g, q_h));
            }
        }
        Err(From::from(format!("No path found from {} to {}", start, end)))
    }
}

fn main() -> Result<()> {
    let opt = Cli::from_args();

    let f = File::open(opt.file)?;
    let reader = BufReader::new(f);
    let graph = MazeGraph::from_reader(reader)?;
    let (start, finish) = ("AA".to_owned(), "ZZ".to_owned());
    println!("Shortest from AA to ZZ is {}", graph.get_shortest_path_length(&start, &finish, false)?);
    println!("Shortest from AA to ZZ (recursive) is {}", graph.get_shortest_path_length(&start, &finish, true)?);
    Ok(())
}