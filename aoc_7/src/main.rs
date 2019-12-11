use std::cmp;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use structopt::StructOpt;
use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use intcode::program::IntcodeProgram;
use intcode::io;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short = "f", parse(from_os_str))]
    file: PathBuf,
    #[structopt(short = "l")]
    lower_phase_setting: usize,
    #[structopt(short = "u")]
    upper_phase_setting: usize,
    #[structopt(short = "g")]
    use_feedback: bool,
}

fn run_amplifier_chain(program: &String, phase_settings: Vec<i64>, use_feedback: bool) -> Result<i64> {
    let mut amplifiers: Vec<IntcodeProgram> = phase_settings.iter()
        .map(|_| IntcodeProgram::from_raw_input(program)).collect::<Result<Vec<IntcodeProgram>>>()?;

    let num_amplifiers = amplifiers.len();
        
    // Connect non-boundary programs with channels
    for i in 0..(num_amplifiers - 1) {
        let (tx, rx): (Sender<i64>, Receiver<i64>) = mpsc::channel();
        amplifiers[i].replace_output(io::ChannelOutputDevice::new(tx));
        amplifiers[i + 1].replace_input(io::ChannelInputDevice::new(rx));
    }

    if use_feedback {
        let (tx, rx): (Sender<i64>, Receiver<i64>) = mpsc::channel();
        amplifiers[num_amplifiers - 1].replace_output(io::ChannelOutputDevice::new(tx));
        amplifiers[0].replace_input(io::ChannelInputDevice::new(rx));
    }

    // Give each amplifier its phase setting
    for (idx, amplifier) in amplifiers.iter_mut().enumerate() {
        amplifier.give_input(phase_settings[idx]);
    }

    // Give the initial input to the first amplifier
    amplifiers[0].give_input(0);

    // Spawn amplifier threads to concurrently compute signal
    let mut threads = vec![];
    let mut idx = 0;
    let result: Arc<Mutex<Option<i64>>> = Arc::new(Mutex::new(None));
    for mut amplifier in amplifiers {
        let res = result.clone();
        threads.push(thread::Builder::new().name(format!("amplifier{}", idx)).spawn(move || {
            amplifier.execute().unwrap();
            if idx == num_amplifiers - 1 {
                *res.lock().unwrap() = Some(amplifier.get_output().unwrap());
            }
        })?);
        idx += 1;
    }

    for thread in threads {
        if let Err(_) = thread.join() {
            return Err(From::from("Amplifier thread panicked"));
        }
    }

    let calculation_result = *result.lock().unwrap();
    calculation_result.ok_or(From::from("No result was calculated"))
}

fn main() -> Result<()> {
    let opt = Cli::from_args();

    let f = File::open(opt.file)?;
    let mut reader = BufReader::new(f);
    let mut contents = String::new();
    reader.read_to_string(&mut contents).unwrap();

    let (lower, upper) = (opt.lower_phase_setting, opt.upper_phase_setting);
    let num_settings = upper - lower + 1;
    let num_inputs_to_try: usize = (1..=num_settings).fold(1, |acc, x| acc * x);
    let mut max_power_found: i64 = std::i64::MIN;

    for input in (0..num_inputs_to_try).map(|mut idx| {
        // Calculate next permutation of phase settings
        let mut options: Vec<usize> = (lower..=upper).collect();
        std::iter::repeat_with(|| { let tmp = idx % options.len(); idx /= options.len(); options.remove(tmp) as i64 })
            .take(5).collect()
    }) {
        max_power_found = cmp::max(max_power_found, run_amplifier_chain(&contents, input, opt.use_feedback)?);
    }

    println!("Max possible power: {}", max_power_found);
    Ok(())
}
