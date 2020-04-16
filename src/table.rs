use std::io::{self, Write};

#[derive(Debug, Copy, Clone)]
pub enum Alignment {
    Left,
    Right,
}

pub fn write_table(mut out: impl Write, columns: &[Alignment], rows: &[Vec<String>])
    -> io::Result<()>
{
    let mut widths = vec![];
    for i in 0 .. columns.len() {
        let max = rows.iter()
            .by_ref()
            .map(|items| {
                if items.len() != columns.len() {
                    panic!("wrong number of columns in row");
                }
                unsafe { items.get_unchecked(i) }.len()
            })
            .max()
            .unwrap();
        widths.push(max);
    }

    for row in rows {
        for ((field, align), width) in row.iter()
                .zip(columns.iter())
                .zip(widths.iter()) {
            match align {
                Alignment::Left => write!(out, "{:width$} ", field, width = width)?,
                Alignment::Right => write!(out, "{:>width$} ", field, width = width)?,
            }
        }
        writeln!(out)?;
    }

    Ok(())
}

pub fn print_table(columns: &[Alignment], rows: &[Vec<String>]) {
    write_table(io::stdout(), columns, rows)
        .expect("I/O error writing to stdout")
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;
    use Alignment::{Left, Right};

    fn v(input: &[&[&str]]) -> Vec<Vec<String>> {
        let mut out = vec![];
        for row in input {
            let mut rowvec = vec![];
            for item in *row {
                rowvec.push((*item).to_owned());
            }
            out.push(rowvec);
        }
        out
    }

    #[test]
    fn table_test() {
        let mut out = vec![];
        write_table(
            Cursor::new(&mut out),
            &[Left, Right, Left],
            &v(&[
                &["ab",   "cd",   "ef"],
                &["ghi",  "jkl",  "mno"],
                &["pqrs", "tuvw", "xyz!"],
                &["test", "",     "pass"],
            ]),
        ).unwrap();
        let s = String::from_utf8(out).unwrap();
        assert_eq!(s,
            "ab     cd ef   \n\
             ghi   jkl mno  \n\
             pqrs tuvw xyz! \n\
             test      pass \n");
    }

    #[test]
    #[should_panic(expected = "wrong number of columns in row")]
    fn test_too_many_columns() {
        write_table(
            io::sink(),
            &[Left, Left, Left],
            &v(&[
                &["foo", "bar"],
                &["a", "b"],
            ]),
        ).unwrap();
    }

    #[test]
    #[should_panic(expected = "wrong number of columns in row")]
    fn test_not_enough_columns() {
        write_table(
            io::sink(),
            &[Left],
            &v(&[
                &["foo", "bar"],
                &["a", "b"],
            ]),
        ).unwrap();
    }
}
