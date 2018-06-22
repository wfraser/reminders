extern crate chrono;
extern crate toml;

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

#[derive(Debug, Default)]
struct Reminders {
    events: HashMap<String, chrono::NaiveDate>,
}

#[derive(Debug)]
enum Error {
    IO(io::Error),
    Toml(toml::de::Error),
    Config(String),
    Datetime(String, chrono::ParseError)
}

fn read_config(path: impl AsRef<Path>) -> Result<Reminders, Error> {
    let mut reminders = Reminders::default();

    let mut f = File::open(path.as_ref()).map_err(Error::IO)?;
    let mut whole_file = vec![];
    f.read_to_end(&mut whole_file).map_err(Error::IO)?;

    let root = toml::from_slice::<toml::Value>(&whole_file).map_err(Error::Toml)?;
    let events = match root {
        toml::Value::Table(mut map) => {
            let entries = match map.remove("reminders") {
                Some(toml::Value::Table(entries)) => entries,
                Some(other) => {
                    return Err(Error::Config(format!(
                                "expected \"reminders\" to be a table, not {:#?}", other)));
                },
                None => {
                    return Err(Error::Config("missing table \"reminders\" at root".into()));
                }
            };

            for key in map.keys() {
                eprintln!("warning: unexpected root-level entry {:?}", key);
            }

            entries
        }
        _ => {
            return Err(Error::Config(format!("expected a table, not {:?}", root)));
        }
    };

    for (name, date_val) in events.into_iter() {
        let date = match date_val {
            toml::Value::Datetime(dt) => {
                chrono::NaiveDate::parse_from_str(&dt.to_string(), "%Y-%m-%d")
                    .map_err(|e| Error::Datetime(name.clone(), e))?
            },
            _ => {
                return Err(Error::Config(format!(
                            "for {:?}, expected a datetime, not {:?}", name, date_val)));
            }
        };

        reminders.events.insert(name, date);
    }

    Ok(reminders)
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
            Error::Toml(e) => eprintln!("Parse error: {}", e),
            Error::Config(e) => eprintln!("Config error: {}", e),
            Error::Datetime(name, e) => eprintln!("Bad date for {:?}: {}", name, e),
        }
        std::process::exit(2);
    });

    println!("{:#?}", config);
}
