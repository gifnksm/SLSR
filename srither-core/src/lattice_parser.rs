// Copyright (c) 2016 srither-core developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! Parsing a lattice strings.

use std::{fmt, iter};
use std::error::Error;
use geom::Point;

/// An error type which is returned from parsing a string into lattice.
#[derive(Copy, Clone, Debug)]
pub struct ParseLatticeError {
    kind: LatticeErrorKind,
}

#[derive(Copy, Clone, Debug)]
enum LatticeErrorKind {
    InvalidLatticePoint,
}

impl Error for ParseLatticeError {
    fn description(&self) -> &str {
        match self.kind {
            LatticeErrorKind::InvalidLatticePoint => "invalid lattice point found in string",
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

/// A parser parsing a string into lattice.
#[derive(Clone, Debug)]
pub struct LatticeParser<'a> {
    mat: &'a [Vec<char>],
    rows: Vec<usize>,
    cols: Vec<usize>,
}

impl<'a> LatticeParser<'a> {
    /// Creates a lattice parser from lines of strings.
    pub fn from_lines(lines: &'a [Vec<char>]) -> Result<LatticeParser<'a>, ParseLatticeError> {
        use self::ParseLatticeError as Error;

        let rows = lines.iter()
                        .enumerate()
                        .filter(|&(_, cs)| cs.iter().any(|&c| c == '+'))
                        .map(|(i, _)| i)
                        .collect::<Vec<_>>();
        let cols = lines[rows[0]]
                       .iter()
                       .enumerate()
                       .filter(|&(_, &c)| c == '+')
                       .map(|(i, _)| i)
                       .collect::<Vec<_>>();

        // check all rows have same lattice points
        for &r in &rows[1..] {
            let cur_rows = lines[r]
                               .iter()
                               .enumerate()
                               .filter(|&(_, &c)| c == '+')
                               .map(|(i, _)| i);

            let count = cur_rows.zip(&cols).filter(|&(p, &q)| p == q).count();
            if count != cols.len() {
                return Err(Error::invalid_lattice_point());
            }
        }

        Ok(LatticeParser {
            mat: lines,
            rows: rows,
            cols: cols,
        })
    }

    /// Returns the number of the rows.
    #[inline]
    pub fn num_rows(&self) -> usize {
        self.rows.len()
    }

    /// Returns the number of the columns.
    #[inline]
    pub fn num_cols(&self) -> usize {
        self.cols.len()
    }

    /// Returns an iterator iterating the vertical edges of the lattice.
    #[inline]
    pub fn v_edges(&self) -> VEdges {
        VEdges::new(self)
    }

    /// Returns an iterator iterating the horizontal edges of the lattice.
    #[inline]
    pub fn h_edges(&self) -> HEdges {
        HEdges::new(self)
    }

    /// Returns an iterator iterating the cells of the lattice.
    #[inline]
    pub fn cells(&self) -> Cells {
        Cells::new(self)
    }
}

/// An iterator iterating the vertical edges of the lattice.
#[derive(Copy, Clone, Debug)]
pub struct VEdges<'a> {
    row: usize,
    col: usize,
    rows: &'a [usize],
    cols: &'a [usize],
    mat: &'a [Vec<char>],
}

impl<'a> VEdges<'a> {
    fn new(parser: &'a LatticeParser) -> VEdges<'a> {
        VEdges {
            row: 0,
            col: 0,
            rows: &parser.rows,
            cols: &parser.cols,
            mat: parser.mat,
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
                continue;
            }

            let (rs, re) = (self.rows[self.row], self.rows[self.row + 1]);
            let c = self.cols[self.col];

            let p = Point(self.row as i32, self.col as i32);
            let s = self.mat[rs + 1..re]
                        .iter()
                        .map(|row| {
                            if c < row.len() {
                                row[c]
                            } else {
                                ' '
                            }
                        })
                        .collect::<String>();
            self.col += 1;
            return Some((p, s));
        }
        None
    }
}

/// An iterator iterating the horizontal edges of the lattice.
#[derive(Copy, Clone, Debug)]
pub struct HEdges<'a> {
    row: usize,
    col: usize,
    rows: &'a [usize],
    cols: &'a [usize],
    mat: &'a [Vec<char>],
}

impl<'a> HEdges<'a> {
    fn new(parser: &'a LatticeParser) -> HEdges<'a> {
        HEdges {
            row: 0,
            col: 0,
            rows: &parser.rows,
            cols: &parser.cols,
            mat: parser.mat,
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
                continue;
            }

            let r = self.rows[self.row];
            let (cs, ce) = (self.cols[self.col], self.cols[self.col + 1]);

            let p = Point(self.row as i32, self.col as i32);
            let s = self.mat[r][cs + 1..ce]
                        .iter()
                        .cloned()
                        .collect::<String>();
            self.col += 1;
            return Some((p, s));
        }
        None
    }
}

/// An iterator iterating the cells of the lattice.
#[derive(Copy, Clone, Debug)]
pub struct Cells<'a> {
    row: usize,
    col: usize,
    rows: &'a [usize],
    cols: &'a [usize],
    mat: &'a [Vec<char>],
}

impl<'a> Cells<'a> {
    fn new(parser: &'a LatticeParser) -> Cells<'a> {
        Cells {
            row: 0,
            col: 0,
            rows: &parser.rows,
            cols: &parser.cols,
            mat: parser.mat,
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
                continue;
            }

            let (rs, re) = (self.rows[self.row], self.rows[self.row + 1]);
            let (cs, ce) = (self.cols[self.col], self.cols[self.col + 1]);

            let p = Point(self.row as i32, self.col as i32);
            let s = self.mat[rs + 1..re]
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
            return Some((p, s));
        }
        None
    }
}
