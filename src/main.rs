#![feature(ascii_char)]
const ENABLE_MOUSE_INPUT: &str = "\x1b[?1003h";
const DISABLE_MOUSE_INPUT: &str = "\x1b[?1003l";
const CSI: &str = "\x1b[";
const EVENT_TRACKING_PREFIX: &str = "\x1b[M";

use std::fs::File;
use std::io::{stdin, Read};
use std::os::fd::{AsRawFd, RawFd};
use std::{error::Error, thread};

use console::Term;
use signal_hook::{consts::SIGINT, consts::SIGTERM, iterator::Signals};
use termios::*;

fn end(lflag_backup: u32, std_fd: RawFd) {
    let mut termios = Termios::from_fd(std_fd).unwrap();
    termios.c_lflag = lflag_backup;
    tcsetattr(std_fd, TCSANOW, &termios).unwrap();
    print!("{}", DISABLE_MOUSE_INPUT);
}

fn read_byte(t: &mut Term) -> std::io::Result<u8> {
    let mut out: [u8; 1] = [0];
    t.read(&mut out)?;
    Ok(out[0])
}

fn main() -> Result<(), Box<dyn Error>> {
    let file = match File::open("/dev/tty") {
        Err(why) => panic!("couldn't open {}: {}", "/dev/tty", why.to_string()),
        Ok(file) => file,
    };
    let std_fd = file.as_raw_fd();

    let mut termios = Termios::from_fd(std_fd).unwrap();
    tcgetattr(std_fd, &mut termios).unwrap();

    //termios.c_lflag |= ECHO; // during debugging, you need to uncomment this a lot

    let lflag_backup = termios.c_lflag.clone();

    termios.c_lflag &= !(ECHO | ECHONL);
    tcsetattr(std_fd, TCSANOW, &termios).unwrap();

    print!("{}", ENABLE_MOUSE_INPUT);

    let mut signals = Signals::new(&[SIGINT, SIGTERM])?;

    thread::spawn(move || {
        for _ in signals.forever() {
            end(lflag_backup, std_fd);
            std::process::exit(0);
        }
    });

    let mut term = Term::stdout();

    'TopLoop: loop {
        let read_data = term.read_char().unwrap_or('\0');
        println!("{}", read_data.as_ascii().unwrap().to_u8());
        println!(
            "{}",
            EVENT_TRACKING_PREFIX.chars().collect::<Vec<char>>()[0]
                .as_ascii()
                .unwrap()
                .to_u8()
        );
        if read_data == EVENT_TRACKING_PREFIX.chars().collect::<Vec<char>>()[0] {
            for chr in &EVENT_TRACKING_PREFIX.chars().collect::<Vec<char>>()[1..] {
                let read_data = term.read_char().unwrap_or('\0');
                if *chr != read_data {
                    continue 'TopLoop;
                }
            }
            let button = read_byte(&mut term)?;
            let x = read_byte(&mut term)?;
            let y = read_byte(&mut term)?;
            println!("Button: {} at ({}, {})", button + 1, x, y);
        }
    }
}
