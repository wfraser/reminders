use chrono;
use std::io::BufRead;
use super::Error;

#[derive(Debug)]
pub struct Reminders {
    pub events: Vec<Event>,
}

#[derive(Debug)]
pub struct Event {
    pub name: String,
    pub date: chrono::NaiveDate,
}

impl Reminders {
    pub fn from_bufread(r: impl BufRead) -> Result<Self, Error> {
        let mut reminders = Reminders { events: vec![] };
        for (line_number, line_result) in r.lines().enumerate() {
            let mut line = line_result.map_err(Error::IO)?;
            if let Some(comment_pos) = line.find('#') {
                line.truncate(comment_pos);
            }
            if line.trim().is_empty() {
                continue;
            }
            let event = Event::from_string(line)
                .map_err(|e| Error::Config(format!("on line {}: {}", line_number + 1, e)))?;
            reminders.events.push(event);
        }
        Ok(reminders)
    }
}

impl Event {
    pub fn from_string(mut s: String) -> Result<Self, String> {
        let colon_pos = s.find(':').ok_or("no ':' found".to_owned())?;

        let date = {
            let date_str = &s[colon_pos + 1 .. ].trim();
            chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                .map_err(|e| format!("bad date: {}", e))?
        };

        s.truncate(colon_pos);

        Ok(Event {
            name: s,
            date,
        })
    }
}
