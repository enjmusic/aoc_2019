use std::cmp;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use structopt::StructOpt;
use intcode::program::IntcodeProgram;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short = "f", parse(from_os_str))]
    file: PathBuf,
}

struct RobotInput {
    sequence: String,
    function_a: String,
    function_b: String,
    function_c: String,
}

#[derive(PartialEq, Copy, Clone)]
enum Dir {
    North = 0, South = 1, East = 2, West = 3
}

impl Dir {
    fn can_apply(&self, loc: (usize, usize), grid: &Vec<Vec<char>>) -> bool {
        let dimensions = (grid[0].len(), grid.len());
        let can_move = match self {
            Dir::North => loc.1 > 0,
            Dir::South => loc.1 < (dimensions.1 - 1),
            Dir::East => loc.0 < (dimensions.0 - 1),
            Dir::West => loc.0 > 0,
        };
        if !can_move { return false }
        let applied = self.apply(loc);
        is_scaffolding(grid[applied.1][applied.0])
    }

    fn apply(&self, loc: (usize, usize)) -> (usize, usize) {
        match self {
            Dir::North => (loc.0, loc.1 - 1),
            Dir::South => (loc.0, loc.1 + 1),
            Dir::East => (loc.0 + 1, loc.1),
            Dir::West => (loc.0 - 1, loc.1),
        }
    }

    fn get_turn_options(d: Dir) -> Vec<(String, Dir)> {
        match d {
            Dir::North => vec![("L".to_owned(), Dir::West), ("R".to_owned(), Dir::East)],
            Dir::South => vec![("L".to_owned(), Dir::East), ("R".to_owned(), Dir::West)],
            Dir::East => vec![("L".to_owned(), Dir::North), ("R".to_owned(), Dir::South)],
            Dir::West => vec![("L".to_owned(), Dir::South), ("R".to_owned(), Dir::North)],
        }
    }
}

fn is_scaffolding(c: char) -> bool {
    c == '#' || c == '^' || c == '<' || c == '>' || c == 'v'
}

// For a list of length l, set any ranges in prev_uncovered as uncovered and the
// rest as covered. Then try to apply to_cover to the list. If to_cover overlaps
// with a previously covered entry, no ranges will be returned. Otherwise, the
// remaining uncovered ranges in the list will be returned.
fn get_uncovered_ranges(
    list_length: usize,
    to_cover: &Vec<(usize, usize)>,
    prev_uncovered: &Vec<(usize, usize)>
) -> Option<Vec<(usize, usize)>> {
    let mut covered = vec![true; list_length];
    for range in prev_uncovered {
        for i in range.0..range.1 { covered[i] = false; }
    }
    for range in to_cover {
        for i in range.0..range.1 {
            if covered[i] { return None }
            covered[i] = true;
        }
    }

    let uncovered_indices = covered.iter().enumerate().filter_map(|(i, c)| {
        if !*c { Some(i) } else { None }
    }).collect::<Vec<usize>>();
    if uncovered_indices.len() == 0 { return Some(vec![]); }

    let mut prev = uncovered_indices[0] - 1; // Dummy value to help with our comparisons
    let mut ranges = vec![vec![]];
    for i in uncovered_indices {
        if i - prev == 1 {
            ranges.last_mut().unwrap().push(i);
        } else {
            ranges.push(vec![i]);
        }
        prev = i;
    }
    
    Some(ranges.iter().map(|members| (members[0], members.last().unwrap() + 1)).collect::<Vec<(usize, usize)>>())
}

// Try to split ranges into ranges of the same length where
// each resulting range contains the same list of actions in
// the path, or None if that task is impossible for these ranges
fn get_as_same_length_if_possible(
    ranges: &Vec<(usize, usize)>,
    path: &Vec<String>,
) -> Option<Vec<(usize, usize)>> {
    // Assume length of pattern will be the length of the shortest range
    let length = ranges.iter().fold(std::usize::MAX, |acc, (l, u)| cmp::min(acc, u - l));
    let first_range = ranges[0];
    let theorized_action_seq = path[first_range.0..first_range.0 + length].to_vec();
    let mut out = vec![];
    for range in ranges {
        for i in (range.0..range.1).step_by(length) {
            if theorized_action_seq != path[i..i + length].to_vec() { return None }
            out.push((i, i + length));
        }
    }
    Some(out)
}

fn get_path(grid: &Vec<Vec<char>>) -> Vec<String> {
    let mut robot_loc = (0, 0);
    let mut robot_dir = Dir::North;
    grid.iter().enumerate().for_each(|(y, row)| row.iter().enumerate().for_each(|(x, tile)| {
        if *tile != '#' && *tile != '.' {
            robot_loc = (x, y);
            robot_dir = match *tile {
                '^' => Dir::North,
                'v' => Dir::South,
                '<' => Dir::West,
                _ => Dir::East // '>'
            };
        }
    }));

    let mut path: Vec<String> = vec![];
    loop {
        let mut traveled_straight = 0;
        while robot_dir.can_apply(robot_loc, &grid) { // Go straight as far as possible
            traveled_straight += 1;
            robot_loc = robot_dir.apply(robot_loc);
        }
        if traveled_straight != 0 { path.push(traveled_straight.to_string()); }

        let mut found_turn = false;
        for (action, option) in Dir::get_turn_options(robot_dir) {
            if option.can_apply(robot_loc, &grid) {
                robot_dir = option;
                path.push(action);
                found_turn = true;
                break
            }
        }

        if !found_turn { break }
    }
    path
}

// For a given path subsequence (either at the beginning or the end of the path),
// get the spots elsewhere in the path where it's repeated and get all combinations
// of those with the original path subsequence included.
fn get_subsequence_repeat_combinations_including_self(range: (usize, usize), path: &Vec<String>) -> Vec<Vec<(usize, usize)>> {
    let subsequence = path[range.0..range.1].to_vec();
    let mut repeats = vec![];
    let range_size = range.1 - range.0;
    for i in range.1..=path.len() - range_size { // Get repeats
        if path[i..i + range_size].to_vec() == subsequence { repeats.push((i, i + range_size)); }
    }

    (0..(1 << repeats.len())).map(|mut mask| { // Get combinations
        let mut curr_combination = vec![];
        for j in 0..repeats.len() {
            if mask & 1 != 0 { curr_combination.push(repeats[j]); }
            mask = mask >> 1;
        }
        curr_combination.push(range);
        curr_combination
    }).collect::<Vec<Vec<(usize, usize)>>>()
}

fn get_function_from_range(path: &Vec<String>, range: (usize, usize)) -> String {
    path[range.0..range.1].to_vec().join(",")
}

fn make_robot_input(
    path: &Vec<String>,
    fn_a_uses: &Vec<(usize, usize)>,
    fn_b_uses: &Vec<(usize, usize)>,
    fn_c_uses: &Vec<(usize, usize)>,
) -> RobotInput {
    let fn_a_uses_with_range_start = fn_a_uses.iter().map(|(l, _)| (*l, 'A'))
    .collect::<Vec<(usize, char)>>();
    let fn_b_uses_with_range_start = fn_b_uses.iter().map(|(l, _)| (*l, 'B'))
        .collect::<Vec<(usize, char)>>();
    let fn_c_uses_with_range_start = fn_c_uses.iter().map(|(l, _)| (*l, 'C'))
        .collect::<Vec<(usize, char)>>();

    // Combine function uses, sort by range start, and transform into sequence
    let mut fn_uses_combined = [
        &fn_a_uses_with_range_start[..],
        &fn_b_uses_with_range_start[..],
        &fn_c_uses_with_range_start[..]
    ].concat();
    fn_uses_combined.sort_by(|(a, _), (b, _)| a.cmp(b));
    let sequence = fn_uses_combined.iter().map(|(_, c)| (*c).to_string())
        .collect::<Vec<String>>().join(",");
        
    RobotInput{
        sequence: sequence,
        function_a: if fn_a_uses.len() == 0 { "".to_owned() } else { get_function_from_range(path, fn_a_uses[0]) },
        function_b: if fn_b_uses.len() == 0 { "".to_owned() } else { get_function_from_range(path, fn_b_uses[0]) },
        function_c: if fn_c_uses.len() == 0 { "".to_owned() } else { get_function_from_range(path, fn_c_uses[0]) },
    }
}

fn get_robot_input(
    path: &Vec<String>,
) -> Option<RobotInput> {
    for fn_a_length in 4..=10 {
        let fn_a_range = (0, fn_a_length);
        if get_function_from_range(path, fn_a_range).len() > 20 { continue }
        for fn_a_ranges in get_subsequence_repeat_combinations_including_self(fn_a_range, path) {
            if let Some(uncovered_after) = get_uncovered_ranges(path.len(), &fn_a_ranges, &vec![(0, path.len())]) {
                if uncovered_after.len() == 0 {
                    if fn_a_ranges.len() <= 10 {
                        // We only need one function
                        return Some(make_robot_input(path, &fn_a_ranges, &vec![], &vec![]));
                    }
                    continue
                }

                // Get candidates for function B in the first uncovered range
                let fn_b_pool = uncovered_after[0];
                for i in 4..=cmp::min(10, fn_b_pool.1 - fn_b_pool.0) {
                    let fn_b_range = (fn_b_pool.0, fn_b_pool.0 + i);
                    if get_function_from_range(path, fn_b_range).len() > 20 { continue }
                    for fn_b_ranges in get_subsequence_repeat_combinations_including_self(fn_b_range, path) {
                        if let Some(uncovered_after) = get_uncovered_ranges(path.len(), &fn_b_ranges, &uncovered_after) {
                            if uncovered_after.len() == 0 {
                                if fn_a_ranges.len() + fn_b_ranges.len() <= 10 {
                                    // We only need two functions
                                    return Some(make_robot_input(path, &fn_a_ranges, &fn_b_ranges, &vec![]));
                                }
                                continue
                            }

                            if let Some(fn_c_ranges) = get_as_same_length_if_possible(&uncovered_after, path) {
                                // Last 2 checks before we can be confident that we have a robot input
                                if get_function_from_range(path, fn_c_ranges[0]).len() > 20 { continue }
                                if fn_a_ranges.len() + fn_b_ranges.len() + fn_c_ranges.len() > 10 { continue }
                                return Some(make_robot_input(path, &fn_a_ranges, &fn_b_ranges, &fn_c_ranges));
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn input_line(program: &mut IntcodeProgram, line: &String) {
    for c in line.chars() { program.give_input((c as u8) as i64); }
    program.give_input(10);
}

fn part1(input: &String) -> Result<Vec<Vec<char>>> {
    let mut program = IntcodeProgram::from_raw_input(input)?;
    program.execute()?;

    let (mut grid, mut curr_line) = (vec![], vec![]);
    for c in program.get_all_output() {
        if c == 10 {
            if curr_line.len() != 0 { grid.push(curr_line.clone()); curr_line = vec![]; };
        } else {
            curr_line.push((c as u8) as char);
        }
    }

    let mut alignment_sum = 0;
    for i in 1..(grid.len() - 1) {
        for j in 1..(grid[0].len() - 1) {
            let inter = vec![Dir::North, Dir::South, Dir::East, Dir::West].iter().all(|d| d.can_apply((j, i), &grid));
            if inter && is_scaffolding(grid[i][j]) { alignment_sum += i * j; }
        }
    }

    println!("Alignment sum: {}", alignment_sum);
    Ok(grid)
}

fn part2(input: &String, grid: &mut Vec<Vec<char>>) -> Result<()> {
    if let Some(robot_input) = get_robot_input(&get_path(&grid)) {
        let mut memory = IntcodeProgram::raw_to_memory(input)?;
        memory[0] = 2;
        let mut program = IntcodeProgram::from_memory(memory);

        // Input main movement routine and functions, decline video feed
        input_line(&mut program, &robot_input.sequence);
        input_line(&mut program, &robot_input.function_a);
        input_line(&mut program, &robot_input.function_b);
        input_line(&mut program, &robot_input.function_c);
        input_line(&mut program, &"n".to_owned());

        program.execute()?;
        if let Some(dust_collected) = program.get_all_output().last() {
            Ok(println!("Dust collected: {}", dust_collected))
        } else {
            Err(From::from("No output for dust collected!"))
        }
    } else {
        Err(From::from("Found no eligible robot input"))
    }
}

fn main() -> Result<()> {
    let opt = Cli::from_args();

    let f = File::open(opt.file)?;
    let mut reader = BufReader::new(f);
    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;

    let mut grid = part1(&contents)?;
    part2(&contents, &mut grid)?;
    Ok(())
}