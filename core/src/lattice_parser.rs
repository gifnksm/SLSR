use std::{iter, fmt};
use std::error::Error;
use geom::Point;

#[derive(Copy, Clone, Debug)]
pub struct ParseLatticeError {
    kind: LatticeErrorKind
}

#[derive(Copy, Clone, Debug)]
enum LatticeErrorKind {
    InvalidLatticePoint
}

impl Error for ParseLatticeError {
    fn description(&self) -> &str {
        match self.kind {
            LatticeErrorKind::InvalidLatticePoint
                => "invalid lattice point found in string"
        }
    }
}

impl fmt::Display for ParseLatticeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(f)
    }
}

impl ParseLatticeError {
    fn invalid_lattice_point() -> ParseLatticeError {
        ParseLatticeError { kind: LatticeErrorKind::InvalidLatticePoint }
    }
}

#[derive(Clone, Debug)]
pub struct LatticeParser<'a> {
    mat: &'a [Vec<char>],
    rows: Vec<usize>,
    cols: Vec<usize>
}

impl<'a> LatticeParser<'a> {
    pub fn from_lines(lines: &'a[Vec<char>])
                      -> Result<LatticeParser<'a>, ParseLatticeError>
    {
        use self::ParseLatticeError as Error;

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

        // check all rows have same lattice points
        for &r in &rows[1..] {
            let cur_rows = lines[r].iter()
                .enumerate()
                .filter(|&(_, &c)| c == '+')
                .map(|(i, _)| i);

            let count = cur_rows.zip(&cols).filter(|&(p, &q)| p == q).count();
            if count != cols.len() {
                return Err(Error::invalid_lattice_point())
            }
        }

        Ok(LatticeParser { mat: lines, rows: rows, cols: cols })
    }

    #[inline]
    pub fn num_rows(&self) -> usize { self.rows.len() }
    #[inline]
    pub fn num_cols(&self) -> usize { self.cols.len() }

    #[inline]
    pub fn v_edges(&self) -> VEdges { VEdges::new(self) }
    #[inline]
    pub fn h_edges(&self) -> HEdges { HEdges::new(self) }
    #[inline]
    pub fn cells(&self) -> Cells { Cells::new(self) }
}

#[derive(Copy, Clone, Debug)]
pub struct VEdges<'a> {
    row: usize, col: usize,
    rows: &'a [usize],
    cols: &'a [usize],
    mat: &'a [Vec<char>]
}

impl<'a> VEdges<'a> {
    fn new(parser: &'a LatticeParser) -> VEdges<'a>
    {
        VEdges {
            row: 0, col: 0,
            rows: &parser.rows, cols: &parser.cols,
            mat: parser.mat
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

#[derive(Copy, Clone, Debug)]
pub struct HEdges<'a> {
    row: usize, col: usize,
    rows: &'a [usize],
    cols: &'a [usize],
    mat: &'a [Vec<char>]
}

impl<'a> HEdges<'a> {
    pub fn new(parser: &'a LatticeParser) -> HEdges<'a>
    {
        HEdges {
            row: 0, col: 0,
            rows: &parser.rows, cols: &parser.cols,
            mat: parser.mat
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

#[derive(Copy, Clone, Debug)]
pub struct Cells<'a> {
    row: usize, col: usize,
    rows: &'a [usize],
    cols: &'a [usize],
    mat: &'a [Vec<char>]
}

impl<'a> Cells<'a> {
    pub fn new(parser: &'a LatticeParser) -> Cells<'a>
    {
        Cells {
            row: 0, col: 0,
            rows: &parser.rows, cols: &parser.cols,
            mat: parser.mat
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
