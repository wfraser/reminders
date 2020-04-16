#[derive(Debug, Copy, Clone)]
pub enum Alignment {
    Left,
    Right,
}

impl Default for Alignment {
    fn default() -> Self {
        Self::Left
    }
}

pub fn print_table(columns: &[Alignment], rows: &[Vec<String>]) {
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
                Alignment::Left => print!("{:width$} ", field, width = width),
                Alignment::Right => print!("{:>width$} ", field, width = width),
            }
        }
        println!();
    }
}
