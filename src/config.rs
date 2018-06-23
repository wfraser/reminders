use chrono;
use serde;
use serde::de::{self, MapAccess, Visitor};
use toml;

#[derive(Debug)]
pub struct Reminders {
    pub events: Vec<Event>,
}

#[derive(Debug)]
pub struct Event {
    pub name: String,
    pub date: chrono::NaiveDate,
}

impl<'de> serde::Deserialize<'de> for Reminders {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D)
        -> Result<Self, D::Error>
    {
        struct RootVisitor;
        impl<'de> Visitor<'de> for RootVisitor {
            type Value = Reminders;

            fn expecting(&self, f: &mut ::std::fmt::Formatter)
                -> ::std::fmt::Result
            {
                f.write_str("a `reminders` table")
            }

            fn visit_map<V: MapAccess<'de>>(self, mut map: V)
                -> Result<Self::Value, V::Error>
            {
                let mut reminders = Err(de::Error::missing_field("reminders"));
                while let Some(key) = map.next_key::<&'de str>()? {
                    if key == "reminders" {
                        let Events(events) = map.next_value()?;
                        reminders = Ok(Reminders { events });
                    } else {
                        eprintln!("warning: unexpected root-level entry {:?}
                                    in configuration", key);
                        map.next_value::<toml::Value>()?; // ignore the value and move on
                    }
                }
                reminders
            }
        }

        deserializer.deserialize_map(RootVisitor)
    }
}

struct Events(Vec<Event>);
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
                    events.push(Event {
                        name: key.to_owned(),
                        date,
                    });
                }

                Ok(Events(events))
            }
        }

        deserializer.deserialize_map(EventsVisitor)
    }
}
