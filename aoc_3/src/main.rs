use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::process;
use std::path::PathBuf;
use std::cmp;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short = "f", parse(from_os_str))]
    file: PathBuf,
}

#[derive(Clone, Copy)]
struct WireSegment {
    x1: i64,
    y1: i64,
    x2: i64,
    y2: i64,
    // How many steps has the wire taken before the start of this one?
    start_step: u64,
}

impl WireSegment {
    // Make a new WireSegment similar to the input but with
    //  x1,x2 and y1,y2 always going in increasing order
    fn to_increasing_order(input: WireSegment) -> WireSegment {
        let mut out = input.clone();
        if out.x1 > out.x2 {
            out.x2 = input.x1;
            out.x1 = input.x2;
        }
        if out.y1 > out.y2 {
            out.y2 = input.y1;
            out.y1 = input.y2;
        }
        out
    }
}

// A wire is composed of a list of segments
fn input_line_to_wire(line: io::Result<String>) -> Vec<WireSegment> {
    let mut curr_x = 0;
    let mut curr_y = 0;
    let mut curr_length: u64 = 0;
    let mut ret: Vec<WireSegment> = vec![];

    for vector in line.unwrap().split(",") {
        // Take first character of vector as direction
        let mut chars = vector.chars();
        let direction = chars.nth(0).unwrap();

        // Slice pointer was advanced by nth(0),
        // parse rest of string as magnitude now
        let magnitude = chars.as_str().parse::<i64>().unwrap();

        let new_x_and_y = match direction {
            'U' => (curr_x, curr_y + magnitude),
            'D' => (curr_x, curr_y - magnitude),
            'L' => (curr_x - magnitude, curr_y),
            'R' => (curr_x + magnitude, curr_y),
            c => {
                println!("Invalid direction character: {}", c);
                process::exit(1);
            },
        };

        ret.push(WireSegment{
            x1: curr_x,
            y1: curr_y,
            x2: new_x_and_y.0,
            y2: new_x_and_y.1,
            start_step: curr_length
        });

        curr_x = new_x_and_y.0;
        curr_y = new_x_and_y.1;
        curr_length += magnitude as u64;
    }

    ret
}

fn get_intersection_manhattan_distance_from_origin(seg_a: WireSegment, seg_b: WireSegment) -> Option<u64> {
    // Convert x1,x2 and y1,y2 to always be increasing in that order
    let seg_a_mod = WireSegment::to_increasing_order(seg_a);
    let seg_b_mod = WireSegment::to_increasing_order(seg_b);

    // No intersection if x ranges of segments A and B don't overlap
    if seg_a_mod.x2 < seg_b_mod.x1 || seg_b_mod.x2 < seg_a_mod.x1 { return None; }

    // No intersection if y ranges of segments A and B don't overlap
    if seg_a_mod.y2 < seg_b_mod.y1 || seg_b_mod.y2 < seg_a_mod.y1 { return None; }

    // Get the range on each axis of where the segments intersect
    let x_intersect_range = if seg_a_mod.x2 <= seg_b_mod.x2 {
        (cmp::max(seg_a_mod.x1, seg_b_mod.x1), cmp::min(seg_a_mod.x2, seg_b_mod.x2))
    } else {
        (cmp::max(seg_b_mod.x1, seg_a_mod.x1), cmp::min(seg_b_mod.x2, seg_a_mod.x2))
    };

    let y_intersect_range = if seg_a_mod.y2 <= seg_b_mod.y2 {
        (cmp::max(seg_a_mod.y1, seg_b_mod.y1), cmp::min(seg_a_mod.y2, seg_b_mod.y2))
    } else {
        (cmp::max(seg_b_mod.y1, seg_a_mod.y1), cmp::min(seg_b_mod.y2, seg_a_mod.y2))
    };

    // TODO: Calculate the best possible combined step length for this intersection
    // Also, make the return value Option<(u64, u64)> where the first item in the
    // tuple is the existing Manhattan distance calculation and the second item is
    // the best possible step length.

    // Get the x, y components of the intersection ranges closest to the origin
    // and add them, ending up with the intersection's minimum manhattan distance
    let min_abs_x_distance = cmp::min(x_intersect_range.0.abs(), x_intersect_range.1.abs()) as u64;
    let min_abs_y_distance = cmp::min(y_intersect_range.0.abs(), y_intersect_range.1.abs()) as u64;
    Some(min_abs_x_distance + min_abs_y_distance)
}

fn main() {
    let opt = Cli::from_args();
    let file = File::open(opt.file.clone());

    if let Err(e) = file {
        println!("Failed to read input file: {}", e);
        process::exit(1);
    }

    let reader = BufReader::new(file.unwrap());
    let wires: Vec<Vec<WireSegment>> = reader.lines().map(input_line_to_wire).collect();

    if wires.len() != 2 {
        println!("Invalid # of wires specified in input file: {}", wires.len());
        process::exit(1);
    }

    // Compare segments between wires A and B for intersections
    // and find the closest non-trivial one to the central port.
    let mut closest_manhattan_distance: u64 = std::u64::MAX;
    for segment_a in wires[0].clone() {
        for segment_b in wires[1].clone() {
            if let Some(manhattan_distance) = get_intersection_manhattan_distance_from_origin(segment_a, segment_b) {
                if manhattan_distance < closest_manhattan_distance && manhattan_distance != 0 {
                    closest_manhattan_distance = manhattan_distance;
                }
            }
        }
    }

    println!("Closest non-trivial intersection to central port has Manhattan distance of {}", closest_manhattan_distance);
}
