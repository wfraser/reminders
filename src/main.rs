extern crate chrono;
extern crate serde;
extern crate toml;

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use serde::de::{self, MapAccess, Visitor};

#[derive(Debug, Default)]
struct Reminders {
    events: Vec<(String, chrono::NaiveDate)>,
}

impl<'de> serde::Deserialize<'de> for Reminders {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct RootVisitor;
        impl<'de> Visitor<'de> for RootVisitor {
            type Value = Reminders;
            fn expecting(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                f.write_str("a 'reminders' table")
            }
            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Self::Value, V::Error> {
                let mut reminders = Err(de::Error::missing_field("reminders"));
                while let Some(key) = map.next_key::<&'de str>()? {
                    if key == "reminders" {
                        let Events(events) = map.next_value()?;
                        reminders = Ok(Reminders { events });
                    } else {
                        eprintln!("warning: unexpected root-level entry {:?} in configuration", key);
                        map.next_value::<toml::Value>()?; // ignore the value and move on
                    }
                }
                reminders
            }
        }
        deserializer.deserialize_map(RootVisitor)
    }
}

struct Events(Vec<(String, chrono::NaiveDate)>);
impl<'de> serde::Deserialize<'de> for Events {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct EventsVisitor;
        impl<'de> Visitor<'de> for EventsVisitor {
            type Value = Events;
            fn expecting(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                f.write_str("a table of event names and dates")
            }
            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Self::Value, V::Error> {
                let mut events = vec![];
                while let Some((key, value)) = map.next_entry::<&'de str, toml::Value>()? {
                    let date = match value {
                        toml::Value::Datetime(dt) => {
                            match chrono::NaiveDate::parse_from_str(&dt.to_string(), "%Y-%m-%d") {
                                Ok(date) => date,
                                Err(e) => {
                                    return Err(de::Error::custom(format!(
                                        "invalid date: {}, for `{}`", e, key)));
                                }
                            }
                        },
                        other => {
                            return Err(de::Error::custom(format!(
                                "expected a datetime, not a {} for `{}`", other.type_str(), key)));
                        }
                    };
                    events.push((key.to_owned(), date));
                }

                Ok(Events(events))
            }
        }
        deserializer.deserialize_map(EventsVisitor)
    }
}

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
    for (mut name, date) in config.events {
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
