use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use std::collections::{HashMap, VecDeque};
use structopt::StructOpt;
use intcode::program::{Event, IntcodeProgram};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short = "f", parse(from_os_str))]
    file: PathBuf,
}

fn run(input: &String) -> Result<()> {
    let memory = IntcodeProgram::raw_to_memory(input)?;
    let mut programs = (0..50).map(|addr| {
        let mut program = IntcodeProgram::from_memory(memory.clone());
        program.give_input(addr as i64);
        program
    }).collect::<Vec<IntcodeProgram>>();

    let mut buffered_out: Vec<Vec<i64>> = vec![vec![]; 50];
    let mut ready_packets: HashMap<usize, VecDeque<(i64, i64)>> = HashMap::new();
    let (mut last_nat_packet, mut nat_packet): (Option<(i64, i64)>, Option<(i64, i64)>) = (None, None);
    loop {
        let mut idle = true;
        for (idx, program) in programs.iter_mut().enumerate() {
            match program.execute_until_event()? {
                Event::Exited => () /* An exited program will continue to emit this event */,
                Event::InputRequired => {
                    let entry = ready_packets.entry(idx).or_insert(VecDeque::new());
                    if let Some((x, y)) = entry.pop_back() {
                        idle = false;
                        program.give_input(x);
                        program.give_input(y);
                    } else {
                        program.give_input(-1);
                    }
                },
                Event::ProducedOutput => {
                    idle = false;
                    buffered_out[idx].push(program.get_output().unwrap());
                    if buffered_out[idx].len() == 3 {
                        let (addr, x, y) = (buffered_out[idx][0] as usize, buffered_out[idx][1], buffered_out[idx][2]);
                        buffered_out[idx].clear();
                        if addr < 50 {
                            let entry = ready_packets.entry(addr).or_insert(VecDeque::new());
                            entry.push_front((x, y));
                        } else if addr == 255 {
                            nat_packet = Some((x, y));
                        } else {
                            return Err(From::from(format!("Invalid addr: {}", addr)));
                        }
                    }
                },
            }
        }

        if idle && nat_packet.is_some() {
            let packet = nat_packet.unwrap();
            if last_nat_packet.is_some() && packet.1 == last_nat_packet.unwrap().1 {
                println!("First repeated Y value sent by NAT: {}", packet.1);
                break
            } else if last_nat_packet.is_none() {
                println!("Y value of first packet sent to NAT: {}", packet.1);
            }

            let entry = ready_packets.entry(0).or_insert(VecDeque::new());
            entry.push_front(packet);
            last_nat_packet = nat_packet;
            nat_packet = None;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let opt = Cli::from_args();

    let f = File::open(opt.file)?;
    let mut reader = BufReader::new(f);
    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;

    run(&contents)
}
