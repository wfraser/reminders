#![deny(rust_2018_idioms)]

mod config;
use config::{Event, Reminders};

mod table;
use table::Alignment;

use std::cmp::Ordering;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;

use calendar_duration::CalendarDurationExt;

const REVERSE: &str = "\x1b[7m";
const RESET: &str = "\x1b[0m";

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

fn main() {
    let path = config_path().unwrap_or_else(|| {
        eprintln!("usage: {} <reminders.conf>", std::env::args().next().unwrap());
        std::process::exit(2);
    });

    let config = read_config(&path).unwrap_or_else(|e| {
        eprint!("error reading config file {path:?}: ");
        match e {
            Error::IO(e) => eprintln!("I/O error: {e}"),
            Error::Config(e) => eprintln!("{e}"),
        }
        std::process::exit(2);
    });

    let now = chrono::Local::now().date_naive();

    let columns = vec![
        Alignment::None,    // fmt_prefix
        Alignment::Left,    // name
        Alignment::Right,   // date
        Alignment::Right,   // years
        Alignment::Left,    // "year(s),"
        Alignment::Right,   // months
        Alignment::Left,    // "month(s),"
        Alignment::Right,   // days
        Alignment::Left,    // "day(s)"
        Alignment::Left,    // "ago / to go"
        Alignment::Left,    // "(1234 days)"
        Alignment::None,    // fmt_suffix
    ];

    let mut output = vec![];
    for Event { mut name, date } in config.events {
        name.push(':');
        let total_days = (now - date).num_days();

        let formatted_date = date.format("%B %-d, %Y -");

        let diff = now.calendar_duration_from(date);
        let (years, mut years_unit) = match diff.years {
            0 => (String::new(), String::new()),
            1 => ("1".to_owned(), "year".to_owned()),
            y => (y.to_string(), "years".to_owned()),
        };
        let (months, mut months_unit) = match diff.months {
            0 => (String::new(), String::new()),
            1 => ("1".to_owned(), "month".to_owned()),
            m => (m.to_string(), "months".to_owned()),
        };
        let (days, days_unit) = match diff.days {
            0 => (String::new(), String::new()),
            1 => ("1".to_owned(), "day".to_owned()),
            d => (d.to_string(), "days".to_owned()),
        };

        if !days.is_empty() {
            if !months_unit.is_empty() {
                months_unit.push(',');
            }
            if !years_unit.is_empty() {
                years_unit.push(',');
            }
        } else if !months.is_empty() && !years_unit.is_empty() {
            years_unit.push(',');
        }

        let suffix = match total_days.cmp(&0) {
            Ordering::Less => "to go",
            Ordering::Equal => "",
            Ordering::Greater => "ago",
        }.to_owned();

        let total_days = match total_days {
            0 => "(today)".to_owned(),
            1 => "(yesterday)".to_owned(),
            -1 => "(tomorrow)".to_owned(),
            d => format!("({d} days)"),
        };

        let (fmt_prefix, fmt_suffix) = match diff.months {
            0 | 11 => (REVERSE.to_owned(), RESET.to_owned()),
            _ => (String::new(), String::new())
        };

        output.push(vec![
            fmt_prefix,
            name,
            formatted_date.to_string(),
            years,
            years_unit,
            months,
            months_unit,
            days,
            days_unit,
            suffix,
            total_days,
            fmt_suffix,
        ]);
    }

    table::print_table(&columns, &output);
}
