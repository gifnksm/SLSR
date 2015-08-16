use std::iter;
use geom::Point;

pub fn find_lattice(lines: &[Vec<char>]) -> Option<(Vec<usize>, Vec<usize>)> {
    let rows = lines.iter()
        .enumerate()
        .filter(|&(_, cs)| cs.iter().any(|&c| c == '+'))
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    let cols = lines[rows[0]].iter()
        .enumerate()
        .filter(|&(_, &c)| c == '+')
        .map(|(i, _)| i)
        .collect::<Vec<_>>();

    for &r in &rows {
        if lines[r].iter().position(|&c| c == '+') != Some(cols[0]) {
            return None
        }
        if lines[r].iter().rposition(|&c| c == '+') != Some(cols[cols.len() - 1]) {
            return None
        }
        for &c in &cols {
            if lines[r].len() <= c {
                return None
            }
            if lines[r][c] != '+' {
                return None
            }
        }
    }
    Some((rows, cols))
}

pub struct VEdges<'a> {
    row: usize, col: usize,
    rows: &'a [usize],
    cols: &'a [usize],
    mat: &'a [Vec<char>]
}

impl<'a> VEdges<'a> {
    pub fn new(mat: &'a [Vec<char>], rows: &'a [usize], cols: &'a [usize])
           -> VEdges<'a>
    {
        VEdges {
            row: 0, col: 0,
            rows: rows, cols: cols,
            mat: mat
        }
    }
}

impl<'a> Iterator for VEdges<'a> {
    type Item = (Point, String);
    fn next(&mut self) -> Option<(Point, String)> {
        while self.row + 1 < self.rows.len() {
            if self.col >= self.cols.len() {
                self.row += 1;
                self.col = 0;
                continue
            }

            let (rs, re) = (self.rows[self.row], self.rows[self.row + 1]);
            let c = self.cols[self.col];

            let p = Point(self.row as i32, self.col as i32);
            let s = self.mat[rs + 1 .. re]
                .iter()
                .map(|row| if c < row.len() { row[c] } else { ' ' })
                .collect::<String>();
            self.col += 1;
            return Some((p, s))
        }
        None
    }
}

pub struct HEdges<'a> {
    row: usize, col: usize,
    rows: &'a [usize],
    cols: &'a [usize],
    mat: &'a [Vec<char>]
}

impl<'a> HEdges<'a> {
    pub fn new(mat: &'a [Vec<char>], rows: &'a [usize], cols: &'a [usize])
           -> HEdges<'a>
    {
        HEdges {
            row: 0, col: 0,
            rows: rows, cols: cols,
            mat: mat
        }
    }
}

impl<'a> Iterator for HEdges<'a> {
    type Item = (Point, String);
    fn next(&mut self) -> Option<(Point, String)> {
        while self.row < self.rows.len() {
            if self.col + 1 >= self.cols.len() {
                self.col = 0;
                self.row += 1;
                continue
            }

            let r = self.rows[self.row];
            let (cs, ce) = (self.cols[self.col], self.cols[self.col + 1]);

            let p = Point(self.row as i32, self.col as i32);
            let s = self.mat[r][cs + 1 .. ce]
                .iter()
                .cloned()
                .collect::<String>();
            self.col += 1;
            return Some((p, s))
        }
        None
    }
}

pub struct Cells<'a> {
    row: usize, col: usize,
    rows: &'a [usize],
    cols: &'a [usize],
    mat: &'a [Vec<char>]
}

impl<'a> Cells<'a> {
    pub fn new(mat: &'a [Vec<char>], rows: &'a [usize], cols: &'a [usize])
           -> Cells<'a>
    {
        Cells {
            row: 0, col: 0,
            rows: rows, cols: cols,
            mat: mat
        }
    }
}

impl<'a> Iterator for Cells<'a> {
    type Item = (Point, String);
    fn next(&mut self) -> Option<(Point, String)> {
        while self.row + 1 < self.rows.len() {
            if self.col + 1 >= self.cols.len() {
                self.col = 0;
                self.row += 1;
                continue
            }

            let (rs, re) = (self.rows[self.row], self.rows[self.row + 1]);
            let (cs, ce) = (self.cols[self.col], self.cols[self.col + 1]);

            let p = Point(self.row as i32, self.col as i32);
            let s = self.mat[rs + 1 .. re]
                .iter()
                .flat_map(|row| {
                    row.iter()
                        .cloned()
                        .chain(iter::repeat(' '))
                        .skip(cs + 1)
                        .take(ce - cs - 1)
                })
                .collect::<String>();
            self.col += 1;
            return Some((p, s))
        }
        None
    }
}
