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

#[derive(PartialEq, Copy, Clone)]
enum Dir {
    North = 0, South = 1, East = 2, West = 3
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Action {
    Turn(u8), // 0 is left, 1 is right
    Move(usize), // num squares moved
}

impl Action {
    fn stringify(&self) -> String {
        match self {
            Action::Turn(d) => if *d == 0 { "L".to_owned() } else { "R".to_owned() },
            Action::Move(d) => format!("{}", d)
        }
    }
}

impl Dir {
    fn can_apply(&self, loc: (usize, usize), dimensions: (usize, usize), grid: &Vec<Vec<char>>) -> bool {
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

    fn get_all_perpendicular(d: Dir) -> Vec<Dir> {
        match d {
            Dir::North => vec![Dir::West, Dir::East],
            Dir::South => vec![Dir::East, Dir::West],
            Dir::East => vec![Dir::North, Dir::South],
            Dir::West => vec![Dir::South, Dir::North],
        }
    }
}

fn is_scaffolding(c: char) -> bool {
    c == '#' || c == '^' || c == '<' || c == '>' || c == 'v'
}

fn get_remaining_actions(num_actions: usize, a: &Vec<(usize, usize)>, b: &Vec<(usize, usize)>) -> Option<Vec<(usize, usize)>> {
    let mut covered = vec![false; num_actions];
    for range in a {
        for i in range.0..range.1 {
            if covered[i] { return None }
            covered[i] = true;
        }
    }
    for range in b {
        for i in range.0..range.1 {
            if covered[i] { return None }
            covered[i] = true;
        }
    }

    let uncovered_indices = covered.iter().enumerate().filter_map(|(i, c)| {
        if !*c { Some(i) } else { None }
    }).collect::<Vec<usize>>();
    if uncovered_indices.len() == 0 { return Some(vec![]); }

    let mut range_start = uncovered_indices[0];
    if uncovered_indices.len() == 1 { return Some(vec![(range_start, range_start + 1)]); }

    let mut out: Vec<(usize, usize)> = vec![];
    let mut range_len = 1;
    for i in 1..uncovered_indices.len() {
        let curr_idx = uncovered_indices[i];
        if curr_idx - uncovered_indices[i - 1] == 1 {
            range_len += 1;
        } else {
            out.push((range_start, range_start + range_len));
            range_start = curr_idx;
            range_len = 1;
        }
    }
    out.push((range_start, range_start + range_len));
    Some(out)
}

fn gcd(a: usize, b: usize) -> usize {
    let mut params = if a >= b { (a, b) } else { (b, a) };
    while params.1 != 0 {
        params = (params.1, params.0 % params.1);
    }
    params.0
}

fn get_range_combinations_with_extra(ranges: &Vec<(usize, usize)>, extra: (usize, usize)) -> Vec<Vec<(usize, usize)>> {
    let mut out = vec![];

    for i in 0..(2 << ranges.len()) {
        let mut mask = i;
        let mut to_push = vec![];
        for j in 0..ranges.len() {
            if mask & 1 != 0 { to_push.push(ranges[j]); }
            mask = mask >> 1;
        }
        to_push.push(extra);
        out.push(to_push);
    }

    out
}

fn split_to_repeating_if_possible(
    ranges: &Vec<(usize, usize)>,
    actions: &Vec<Action>,
    length: usize
) -> Option<Vec<(usize, usize)>> {
    let mut out = vec![];
    let first_range = ranges[0];
    let theorized_action_seq = actions[first_range.0..first_range.0 + length].to_vec();
    for range in ranges {
        for i in (range.0..range.1).step_by(length) {
            if theorized_action_seq != actions[i..i + length].to_vec() { return None }
            out.push((i, i + length));
        }
    }
    Some(out)
}

struct RobotInput {
    sequence: String,
    function_a: Vec<Action>,
    function_b: Vec<Action>,
    function_c: Vec<Action>,
}

fn get_three_action_sequences(
    actions: &Vec<Action>,
    end_substring_occurrences: &Vec<Vec<(usize, usize)>>,
    start_substring_occurrences: &Vec<Vec<(usize, usize)>>,
) -> Option<RobotInput> {
    for (i, e) in end_substring_occurrences.iter().enumerate() {
        if e.len() == 0 { continue }
        // TODO: filter out sequences whose strings are too long
        let end_range = (actions.len() - (i + 4), actions.len());
        for (j, s) in start_substring_occurrences.iter().enumerate() {
            if s.len() == 0 { continue }
            // TODO: filter out sequences whose strings are too long
            let start_range = (0, j + 4);
            for end_combo in get_range_combinations_with_extra(e, end_range) {
                for start_combo in get_range_combinations_with_extra(s, start_range) {
                    if let Some(remaining_ranges) = get_remaining_actions(actions.len(), &end_combo, &start_combo) {
                        if remaining_ranges.len() == 0 { continue }
                        let mut gcd_rem = remaining_ranges[0].1 - remaining_ranges[0].0;
                        for i in 1..remaining_ranges.len() {
                            gcd_rem = gcd(gcd_rem, remaining_ranges[i].1 - remaining_ranges[i].0);
                        }
                        if let Some(ranges) = split_to_repeating_if_possible(&remaining_ranges, &actions, gcd_rem) {
                            let pattern1_range = start_combo[0];
                            let pattern2_range = end_combo[0];
                            let pattern3_range = ranges[0];

                            if pattern3_range.1 - pattern3_range.0 > 10 {
                                continue // Pattern 3 too long
                            }

                            if start_combo.len() + end_combo.len() + ranges.len() > 10 {
                                continue // Total sequence too long
                            }

                            let pattern1_by_start = start_combo.iter().map(|(l, _)| (*l, 'A')).collect::<Vec<(usize, char)>>();
                            let pattern2_by_start = end_combo.iter().map(|(l, _)| (*l, 'B')).collect::<Vec<(usize, char)>>();
                            let pattern3_by_start = ranges.iter().map(|(l, _)| (*l, 'C')).collect::<Vec<(usize, char)>>();
                            let mut combined = [&pattern1_by_start[..], &pattern2_by_start[..], &pattern3_by_start[..]].concat().to_vec();
                            combined.sort_by(|(a, _), (b, _)| a.cmp(b));

                            return Some(RobotInput{
                                sequence: combined.iter().map(|(_, c)| (*c).to_string()).collect::<Vec<String>>().join(","),
                                function_a: actions[pattern1_range.0..pattern1_range.1].to_vec(),
                                function_b: actions[pattern2_range.0..pattern2_range.1].to_vec(),
                                function_c: actions[pattern3_range.0..pattern3_range.1].to_vec(),
                            });
                        }
                    }
                }
            }
        }
    }

    None
}

fn sequence_to_string(sequence: &Vec<Action>) -> String {
    sequence.iter().map(|f| f.stringify()).collect::<Vec<String>>().join(",")
}

fn input_function(program: &mut IntcodeProgram, function: &Vec<Action>) {
    for c in sequence_to_string(function).chars() {
        program.give_input((c as u8) as i64);
    }
    program.give_input(10);
}

fn part1(input: &String) -> Result<Vec<Vec<char>>> {
    let mut program = IntcodeProgram::from_raw_input(input)?;
    program.execute()?;
    let mut grid = vec![vec![]];
    for c in program.get_all_output() {
        if c == 10 {
            grid.push(vec![]);
        } else {
            grid.last_mut().unwrap().push((c as u8) as char);
        }
    }

    for i in (0..grid.len()).rev() {
        if grid[i].len() == 0 { grid.pop(); } else { break }
    }

    let mut alignment_sum = 0;
    for i in 1..(grid.len() - 1) {
        for j in 1..(grid[0].len() - 1) {
            let mut is_intersection = is_scaffolding(grid[i][j]);
            is_intersection = is_intersection && is_scaffolding(grid[i+1][j]);
            is_intersection = is_intersection && is_scaffolding(grid[i-1][j]);
            is_intersection = is_intersection && is_scaffolding(grid[i][j+1]);
            is_intersection = is_intersection && is_scaffolding(grid[i][j-1]);
            if is_intersection { alignment_sum += i * j; }
        }
    }

    println!("Alignment sum: {}", alignment_sum);
    Ok(grid)
}

fn part2(input: &String, grid: &mut Vec<Vec<char>>) -> Result<()> {
    let dimensions = (grid[0].len(), grid.len());
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

    grid[robot_loc.1][robot_loc.0] = '#';
    let mut num_in_dir = 0;
    let mut actions: Vec<Action> = vec![];
    loop {
        if robot_dir.can_apply(robot_loc, dimensions, &grid) {
            num_in_dir += 1;
            robot_loc = robot_dir.apply(robot_loc);
            continue
        }
        let mut found_turn = false;
        for (idx, option) in Dir::get_all_perpendicular(robot_dir).iter().enumerate() {
            if option.can_apply(robot_loc, dimensions, &grid) {
                if num_in_dir != 0 { actions.push(Action::Move(num_in_dir)) };
                num_in_dir = 0;
                robot_dir = *option;
                actions.push(Action::Turn(idx as u8));
                found_turn = true;
                break
            }
        }
        if !found_turn {
            actions.push(Action::Move(num_in_dir));
            break
        }
    }

    let min_len = 4;

    let substrings_at_end = (min_len..=10).map(|l| actions[actions.len() - l..actions.len()].to_vec())
        .collect::<Vec<Vec<Action>>>();
    
    let mut end_substring_occurrences = vec![];
    for i in min_len..=10 {
        let mut to_concat = vec![];
        for j in 0..actions.len() - (2 * i) {
            if substrings_at_end[i - min_len] == actions[j..j+i].to_vec() {
                to_concat.push((j, j+i));
            }
        }
        end_substring_occurrences.push(to_concat);
    }

    let substrings_at_start = (min_len..=10).map(|l| actions[0..l].to_vec())
        .collect::<Vec<Vec<Action>>>();

    let mut start_substring_occurrences = vec![];
    for i in min_len..=10 {
        let mut to_concat = vec![];
        for j in i..actions.len() - i {
            if substrings_at_start[i - min_len] == actions[j..j+i].to_vec() {
                to_concat.push((j, j+i));
            }
        }
        start_substring_occurrences.push(to_concat);
    }

    if let Some(robot_input) = get_three_action_sequences(
        &actions,
        &end_substring_occurrences,
        &start_substring_occurrences,
    ) {
        let mut memory = IntcodeProgram::raw_to_memory(input)?;
        memory[0] = 2;
        let mut program = IntcodeProgram::from_memory(memory);

        // Input main movement routine
        for c in robot_input.sequence.chars() {
            program.give_input((c as u8) as i64);
        }
        program.give_input(10);

        // Input functions
        input_function(&mut program, &robot_input.function_a);
        input_function(&mut program, &robot_input.function_b);
        input_function(&mut program, &robot_input.function_c);

        // Decline continuous video feed
        program.give_input(('n' as u8) as i64);
        program.give_input(10);

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