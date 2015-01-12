use std::{cmp, iter};
use std::io::{IoResult, BufferedReader};
use std::ops::{Index, IndexMut};

use geom::{Geom, Point, Size};

pub type Elem = Option<u8>;

static OUTSIDE: Elem = None;

#[derive(Clone, Show)]
pub struct Hint {
    size: Size,
    data: Vec<Elem>
}

impl Hint {
    pub fn new(size: Size, data: Vec<Elem>) -> Hint {
        assert!(size.0 > 0 && size.1 > 0);
        Hint { size: size, data: data }
    }

    pub fn from_reader<R: Reader>(reader: R) -> IoResult<Hint> {
        let mut br = BufferedReader::new(reader);

        let mut column = 0;
        let mut mat = vec![];
        for line in br.lines() {
            let row = try!(line).trim_matches('\n').chars().map(|c| {
                match c {
                    '0' => Some(0),
                    '1' => Some(1),
                    '2' => Some(2),
                    '3' => Some(3),
                    _   => None
                }
            }).collect::<Vec<_>>();

            column = cmp::max(column, row.len());
            mat.push(row);
        }
        for row in mat.iter_mut() {
            let len = row.len();
            if len < column {
                row.extend(iter::repeat(None).take(column - len));
            }
        }
        let row = mat.len();
        Ok(Hint::new(Size(row as i32, column as i32), mat.concat()))
    }
}

impl Geom for Hint {
    fn size(&self) -> Size { self.size }
}

impl Index<Point> for Hint {
    type Output = Elem;

    fn index(&self, p: &Point) -> &Elem {
        if self.contains(*p) {
            &self.data[self.point_to_index(*p)]
        } else {
            &OUTSIDE
        }
    }
}

impl IndexMut<Point> for Hint {
    type Output = Elem;

    fn index_mut(&mut self, p: &Point) -> &mut Elem {
        assert!(self.contains(*p));
        let idx = self.point_to_index(*p);
        &mut self.data[idx]
    }
}

#[cfg(test)]
mod tests {
    use super::Hint;
    use geom::{Geom, Size, Point};

    #[test]
    fn from_reader() {
        let input = "123   

345
";
        let hint = Hint::from_reader(input.as_bytes()).unwrap();
        assert_eq!(Size(3, 6), hint.size());
        assert_eq!(Some(1), hint[Point(0, 0)]);
        assert_eq!(Some(2), hint[Point(0, 1)]);
        assert_eq!(Some(3), hint[Point(0, 2)]);
        assert_eq!(None, hint[Point(0, 3)]);
        assert_eq!(None, hint[Point(0, 4)]);
        assert_eq!(None, hint[Point(0, 5)]);
        assert_eq!(None, hint[Point(1, 0)]);
        assert_eq!(None, hint[Point(1, 1)]);
        assert_eq!(None, hint[Point(1, 2)]);
        assert_eq!(None, hint[Point(1, 3)]);
        assert_eq!(None, hint[Point(1, 4)]);
        assert_eq!(None, hint[Point(1, 5)]);
        assert_eq!(Some(3), hint[Point(2, 0)]);
        assert_eq!(None, hint[Point(2, 1)]);
        assert_eq!(None, hint[Point(2, 2)]);
        assert_eq!(None, hint[Point(2, 3)]);
        assert_eq!(None, hint[Point(2, 4)]);
        assert_eq!(None, hint[Point(2, 5)]);
    }
}
