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
        let colon_pos = s.find(':').ok_or_else(|| "no ':' found".to_owned())?;

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

#[test]
fn test_event_parse() {
    let e = Event::from_string("foo:2018-06-25".to_owned()).unwrap();
    assert_eq!("foo", &e.name);
    assert_eq!(chrono::NaiveDate::from_ymd(2018, 6, 25), e.date);
}

#[test]
fn test_event_parse_trim() {
    let e = Event::from_string("  foo  :  2018-06-25  ".to_owned()).unwrap();
    assert_eq!("  foo  ", &e.name);
    assert_eq!(chrono::NaiveDate::from_ymd(2018, 6, 25), e.date);
}

#[test]
fn test_reminders() {
    use std::io::{BufReader, Cursor};
    let input = r"a:2018-06-25
# comment line
b: 9999-12-31 # spaghetti
";
    let r = Reminders::from_bufread(BufReader::new(Cursor::new(input))).unwrap();
    assert_eq!("a", &r.events[0].name);
    assert_eq!(chrono::NaiveDate::from_ymd(2018, 6, 25), r.events[0].date);
    assert_eq!("b", &r.events[1].name);
    assert_eq!(chrono::NaiveDate::from_ymd(9999, 12, 31), r.events[1].date);
    assert_eq!(2, r.events.len());
}
