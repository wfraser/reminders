extern crate chrono;
extern crate serde;
extern crate toml;

mod config;
use config::{Event, Reminders};

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

#[derive(Debug)]
enum Error {
    IO(io::Error),
    Config(toml::de::Error),
}

fn read_config(path: impl AsRef<Path>) -> Result<Reminders, Error> {
    let mut f = File::open(path.as_ref()).map_err(Error::IO)?;
    let mut whole_file = vec![];
    f.read_to_end(&mut whole_file).map_err(Error::IO)?;

    let reminders = toml::from_slice::<Reminders>(&whole_file).map_err(Error::Config)?;
    Ok(reminders)
}

fn daysto(duration: &chrono::Duration) -> String {
    let days = duration.num_days();
    if days == 0 {
        "today".to_owned()
    } else if days == -1 {
        "yesterday".to_owned()
    } else if days == 1 {
        "tomorrow".to_owned()
    } else {
        format!("{} days", days)
    }
}

fn main() {
    let path = std::env::args_os().nth(1).unwrap_or_else(|| {
        eprintln!("usage: {} <reminders.toml>", std::env::args().nth(0).unwrap());
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
        let days = (diff.num_days() as f64).abs();

        let formatted_date = Some(format!("{} -", date.format("%B %-d, %Y")));
        let years = if days > 365.25 {
            Some(format!("{} years,", (days / 365.25).floor()))
        } else {
            None
        };
        let months = if days > (365.25 / 12.) {
            Some(format!("{} months,", ((days % 365.25) / (365.25 / 12.)).floor()))
        } else {
            None
        };
        let just_days = Some(format!("{} days", (days % (365.25 / 12.)).floor()));
        let suffix = if diff.num_days() < -1 {
            Some("to go".to_owned())
        } else if diff.num_days() > 1 {
            Some("ago".to_owned())
        } else {
            None
        };
        let total_days = Some(format!("({})", daysto(&diff)));

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
