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

struct IntersectionRange {
    x_bounds: (i64, i64),
    y_bounds: (i64, i64),
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

    // Precondition: point is a point on the WireSegment `self`
    fn step_length_at_point(&self, point: (i64, i64)) -> u64 {
        let abs_distance = (point.0 - self.x1).abs() + (point.1 - self.y1).abs();
        self.start_step + (abs_distance as u64)
    }

    fn get_intersection_range(&self, other: &WireSegment) -> Option<IntersectionRange> {
        // Convert x1,x2 and y1,y2 to always be increasing in that order
        let seg_a_mod = WireSegment::to_increasing_order(*self);
        let seg_b_mod = WireSegment::to_increasing_order(*other);

        // No intersection if x ranges of segments A and B don't overlap
        if seg_a_mod.x2 < seg_b_mod.x1 || seg_b_mod.x2 < seg_a_mod.x1 { return None; }

        // No intersection if y ranges of segments A and B don't overlap
        if seg_a_mod.y2 < seg_b_mod.y1 || seg_b_mod.y2 < seg_a_mod.y1 { return None; }

        // Get the range on each axis of where the segments intersect
        Some(IntersectionRange{
            x_bounds: if seg_a_mod.x2 <= seg_b_mod.x2 {
                (cmp::max(seg_a_mod.x1, seg_b_mod.x1), cmp::min(seg_a_mod.x2, seg_b_mod.x2))
            } else {
                (cmp::max(seg_b_mod.x1, seg_a_mod.x1), cmp::min(seg_b_mod.x2, seg_a_mod.x2))
            },
            y_bounds: if seg_a_mod.y2 <= seg_b_mod.y2 {
                (cmp::max(seg_a_mod.y1, seg_b_mod.y1), cmp::min(seg_a_mod.y2, seg_b_mod.y2))
            } else {
                (cmp::max(seg_b_mod.y1, seg_a_mod.y1), cmp::min(seg_b_mod.y2, seg_a_mod.y2))
            },
        })
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

fn get_best_intersection_values(seg_a: WireSegment, seg_b: WireSegment) -> Option<(u64, u64)> {
    let intersection_range = seg_a.get_intersection_range(&seg_b);
    if intersection_range.is_none() { return None; }
    let intersection_range = intersection_range.unwrap();

    // Only the min/max bound points of the intersection range matter for calculating
    // the best possible combined step length, nothing in between (proof left to the reader).
    let intersection_bound1 = (
        cmp::min(intersection_range.x_bounds.0, intersection_range.x_bounds.1),
        cmp::min(intersection_range.y_bounds.0, intersection_range.y_bounds.1)
    );

    let intersection_bound2 = (
        cmp::max(intersection_range.x_bounds.0, intersection_range.x_bounds.1),
        cmp::max(intersection_range.y_bounds.0, intersection_range.y_bounds.1)
    );

    let bound1_step_length = seg_a.step_length_at_point(intersection_bound1) + seg_b.step_length_at_point(intersection_bound1);
    let bound2_step_length = seg_a.step_length_at_point(intersection_bound2) + seg_b.step_length_at_point(intersection_bound2);

    // Get the x, y components of the intersection ranges closest to the origin
    // and add them, ending up with the intersection's minimum manhattan distance
    let min_abs_x_distance = cmp::min(intersection_range.x_bounds.0.abs(), intersection_range.x_bounds.1.abs()) as u64;
    let min_abs_y_distance = cmp::min(intersection_range.y_bounds.0.abs(), intersection_range.y_bounds.1.abs()) as u64;

    Some((min_abs_x_distance + min_abs_y_distance, cmp::min(bound1_step_length, bound2_step_length)))
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
    
    let mut closest_manhattan_distance: u64 = std::u64::MAX;
    let mut shortest_step_length: u64 = std::u64::MAX;
    for segment_a in wires[0].clone() {
        for segment_b in wires[1].clone() {
            if let Some((manhattan_distance, min_step_length)) = get_best_intersection_values(segment_a, segment_b) {
                if manhattan_distance < closest_manhattan_distance && manhattan_distance != 0 {
                    closest_manhattan_distance = manhattan_distance;
                }
                if min_step_length < shortest_step_length && min_step_length != 0 {
                    shortest_step_length = min_step_length;
                }
            }
        }
    }

    println!("Closest (by Manhattan distance) non-trivial intersection to central port has Manhattan distance of {}", closest_manhattan_distance);
    println!("Closest (by step length) non-trivial intersection to central port has step length of {}", shortest_step_length);
}
