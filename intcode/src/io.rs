use std::io::{self, prelude::*};
use std::collections::VecDeque;
use std::sync::mpsc::{Sender, Receiver};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub trait IODevice {
    fn put(&mut self, output: i64);
    fn get(&mut self) -> Result<i64>;
}

pub struct DefaultInputDevice {
    buffer: VecDeque<i64>
}

pub struct DefaultOutputDevice {
    buffer: VecDeque<i64>
}

pub struct ChannelInputDevice {
    // Buffer is used if available, otherwise channel is
    buffer: VecDeque<i64>,
    channel: Receiver<i64>,
}

pub struct ChannelOutputDevice {
    // If the output channel is closed we'll write to buffer instead
    buffer: VecDeque<i64>,
    channel: Sender<i64>,
}

impl DefaultInputDevice {
    pub fn new() -> Box<DefaultInputDevice> {
        Box::new(DefaultInputDevice{ buffer: VecDeque::new() })
    }
}

impl DefaultOutputDevice {
    pub fn new() -> Box<DefaultOutputDevice> {
        Box::new(DefaultOutputDevice{ buffer: VecDeque::new() })
    }
}

impl ChannelInputDevice {
    pub fn new(channel: Receiver<i64>) -> Box<ChannelInputDevice> {
        Box::new(ChannelInputDevice{ buffer: VecDeque::new(), channel: channel })
    }
}

impl ChannelOutputDevice {
    pub fn new(channel: Sender<i64>) -> Box<ChannelOutputDevice> {
        Box::new(ChannelOutputDevice{ buffer: VecDeque::new(), channel: channel })
    }
}

// Private

impl IODevice for DefaultInputDevice {
    fn put(&mut self, output: i64) { self.buffer.push_front(output) }
    fn get(&mut self) -> Result<i64> {
        self.buffer.pop_back().map_or_else(|| {
            print!("Enter program input: ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            Ok(input.trim().parse::<i64>()?)
        }, |v| Ok(v))
    }
}

impl IODevice for DefaultOutputDevice {
    fn put(&mut self, output: i64) { self.buffer.push_front(output) }
    fn get(&mut self) -> Result<i64> {
        self.buffer.pop_back().map_or(Err(From::from("No output available")), |x| Ok(x))
    }
}

impl IODevice for ChannelInputDevice {
    fn put(&mut self, output: i64) { self.buffer.push_front(output) }
    fn get(&mut self) -> Result<i64> {
        self.buffer.pop_back().map_or_else(|| self.channel.recv()
            .map_err(|_| From::from("Failed to recv")), |x| Ok(x))
    }
}

impl IODevice for ChannelOutputDevice {
    fn put(&mut self, output: i64) {
        self.channel.send(output).unwrap_or_else(|_| self.buffer.push_front(output))
    }
    fn get(&mut self) -> Result<i64> {
        self.buffer.pop_back().map_or(Err(From::from("No output available")), |x| Ok(x))
    }
}