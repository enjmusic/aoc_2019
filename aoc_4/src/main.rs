use structopt::StructOpt;

type Result<T> = ::std::result::Result<T, Box<dyn(::std::error::Error)>>;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short = "l")]
    lower: u64,
    #[structopt(short = "u")]
    upper: u64,
    #[structopt(short = "i")]
    isolated_repeat_length: Option<u64>,
}

fn digits(i: u64) -> Result<Vec<u8>> {
    i.to_string().chars().map(|x| {
        if let Some(d) = x.to_digit(10) {
            Ok(d as u8)
        } else {
            Err(From::from("Invalid digit"))
        }
    }).collect()
}

fn is_valid_repeat_length(length: u64, isolated_repeat_length: Option<u64>) -> bool {
    if let Some(desired) = isolated_repeat_length {
        length == desired
    } else {
        length > 1
    }
}

fn is_possible_password(number: u64, isolated_repeat_length: Option<u64>) -> bool {
    let digits = digits(number).unwrap();
    let mut found_repeat = false;
    let mut curr_repeat_length = 1;

    for i in 1..digits.len() {
        if digits[i] < digits[i - 1] { return false }
        if digits[i] == digits[i - 1] {
            curr_repeat_length += 1;
        } else {
            found_repeat = found_repeat || is_valid_repeat_length(curr_repeat_length, isolated_repeat_length);
            curr_repeat_length = 1;
        }
    }

    found_repeat || is_valid_repeat_length(curr_repeat_length, isolated_repeat_length)
}

fn main() -> Result<()> {
    let opt = Cli::from_args();

    if opt.lower > opt.upper {
        return Err(From::from("Lower bound must be smaller than upper bound"));
    }

    if digits(opt.lower)?.len() < digits(opt.lower)?.len() {
        return Err(From::from("Bounds must have same # of digits"));
    }

    let num_possible_passwords = (opt.lower..=opt.upper).fold(0, |a, b| {
        a + is_possible_password(b, opt.isolated_repeat_length) as i64
    });

    println!("# possible passwords: {}", num_possible_passwords);
    Ok(())
}
