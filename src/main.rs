extern crate chrono;

mod config;
use config::{Event, Reminders};

use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    Config(String),
}

fn read_config(path: impl AsRef<Path>) -> Result<Reminders, Error> {
    Reminders::from_bufread(
        BufReader::new(
            File::open(path.as_ref()).map_err(Error::IO)?
    ))
}

fn config_path() -> Option<OsString> {
    use std::ffi::OsStr;
    let args = std::env::args_os().skip(1);
    let mut double_dash = false;
    for s in args {
        if !double_dash {
            if s == OsStr::new("--") {
                double_dash = true;
                continue;
            } else if s == OsStr::new("--help") || s == OsStr::new("-h") {
                return None;
            }
        }
        return Some(s);
    }
    None
}

fn duration_ymd(diff: &chrono::Duration) -> (u64, u64, u64) {
    let days = (diff.num_days() as f64).abs();

    let years = if days > 365.25 {
        (days / 365.25).floor() as u64
    } else {
        0
    };

    let months = if days > (365.25 / 12.) {
        ((days % 365.25) / (365.25 / 12.)).floor() as u64
    } else {
        0
    };

    let days = (days % (365.25 / 12.)).floor() as u64;

    (years, months, days)
}

fn main() {
    let path = config_path().unwrap_or_else(|| {
        eprintln!("usage: {} <reminders.conf>", std::env::args().nth(0).unwrap());
        std::process::exit(2);
    });

    let config = read_config(&path).unwrap_or_else(|e| {
        eprint!("error reading config file {:?}: ", path);
        match e {
            Error::IO(e) => eprintln!("I/O error: {}", e),
            Error::Config(e) => eprintln!("{}", e),
        }
        std::process::exit(2);
    });

    let now = chrono::Local::today().naive_local();

    let mut output = vec![];
    for Event { mut name, date } in config.events {
        name.push(':');
        let diff = now - date;
        let days = diff.num_days();

        let formatted_date = Some(format!("{} -", date.format("%B %-d, %Y")));

        let (y,m,d) = duration_ymd(&diff);
        let years = match y {
            0 => None,
            1 => Some("1 year, ".to_owned()),   // significant space
            y => Some(format!("{} years,", y)),
        };
        let months = match m {
            0 => None,
            1 => Some("1 month, ".to_owned()),  // significant space
            m => Some(format!("{} months,", m)),
        };
        let just_days = match d {
            0 => None,
            1 => Some("1 day ".to_owned()),     // significant space
            d => Some(format!("{} days", d)),
        };

        let suffix = if days < 0 {
            Some("to go".to_owned())
        } else if days > 0 {
            Some("ago".to_owned())
        } else {
            None
        };
        let total_days = Some(match days {
            0 => "(today)".to_owned(),
            1 => "(yesterday)".to_owned(),
            -1 => "(tomorrow)".to_owned(),
            d => format!("({} days)", d),
        });

        output.push(vec![
            Some(name),
            formatted_date,
            years,
            months,
            just_days,
            suffix,
            total_days,
        ]);
    }

    let mut widths = vec![];
    for i in 0 .. output[0].len() {
        let max = output.iter()
            .by_ref()
            .map(|items| {
                items[i].as_ref()
                    .map(|s| s.len())
                    .unwrap_or(0)
            })
            .max()
            .unwrap();
        widths.push(max);
    }

    for line in output {
        for (i, field) in line.iter().enumerate() {
            if let Some(field) = field {
                if i >= 1 && i <= 4 {
                    print!("{:>width$} ", field, width = widths[i]);
                } else {
                    print!("{:width$} ", field, width = widths[i]);
                }
            } else {
                print!("{:width$} ", "", width = widths[i]);
            }
        }
        println!();
    }
}
