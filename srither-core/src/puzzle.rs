// Copyright (c) 2016 srither-core developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! Slither link puzzle data structure.

use std::error::Error;
use std::fmt;

use geom::{Geom, Point, Size, Table};
use lattice_parser::ParseLatticeError;

/// A hint of the slither link puzzle.
pub type Hint = Option<u8>;

/// A cell's side.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Side {
    /// A cell is inside.
    In,
    /// A cell is outside.
    Out,
}

/// An edge between cells.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Edge {
    /// An edge between cells is line (different side).
    Line,
    /// An edge between cells is cross (same side).
    Cross,
}

/// Slither link puzzle data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Puzzle {
    size: Size,
    hint: Table<Hint>,
    side: Table<Option<Side>>,
    edge_v: Table<Option<Edge>>,
    edge_h: Table<Option<Edge>>,
    sum_of_hint: u32,
}

impl Puzzle {
    /// Creates an empty slither link puzzle.
    #[inline]
    pub fn new(size: Size) -> Puzzle {
        assert!(size.0 > 0 && size.1 > 0);
        let hint = vec![None; (size.0 * size.1) as usize];
        let side = vec![None; (size.0 * size.1) as usize];
        let edge_v = vec![None; (size.0 * (size.1 + 1)) as usize];
        let edge_h = vec![None; ((size.0 + 1) * size.1) as usize];
        Puzzle::with_data(size, hint, side, edge_v, edge_h)
    }

    #[inline]
    fn with_data(size: Size,
                 hint: Vec<Hint>,
                 side: Vec<Option<Side>>,
                 edge_v: Vec<Option<Edge>>,
                 edge_h: Vec<Option<Edge>>)
                 -> Puzzle {
        assert!(size.0 > 0 && size.1 > 0);
        let mut sum_of_hint = 0;
        for &h in &hint {
            if let Some(n) = h {
                sum_of_hint += n as u32;
            }
        }
        let hint = Table::new(size, None, hint);
        let side = Table::new(size, Some(Side::Out), side);
        let edge_v = Table::new(Size(size.0, size.1 + 1), Some(Edge::Cross), edge_v);
        let edge_h = Table::new(Size(size.0 + 1, size.1), Some(Edge::Cross), edge_h);
        Puzzle {
            size: size,
            hint: hint,
            side: side,
            edge_v: edge_v,
            edge_h: edge_h,
            sum_of_hint: sum_of_hint,
        }
    }

    /// Gets a hint at the point.
    #[inline]
    pub fn hint(&self, p: Point) -> Hint {
        self.hint[p]
    }

    /// Sets a hint at the point.
    #[inline]
    pub fn set_hint(&mut self, p: Point, hint: Hint) {
        if let Some(n) = self.hint[p] {
            self.sum_of_hint -= n as u32;
        }
        if let Some(n) = hint {
            self.sum_of_hint += n as u32;
        }
        self.hint[p] = hint;
    }

    /// Gets a side at the point.
    #[inline]
    pub fn side(&self, p: Point) -> Option<Side> {
        self.side[p]
    }

    /// Set a side at the point.
    #[inline]
    pub fn set_side(&mut self, p: Point, side: Option<Side>) {
        self.side[p] = side;
    }

    /// Gets a horizontal edge above the point.
    #[inline]
    pub fn edge_h(&self, p: Point) -> Option<Edge> {
        self.edge_h[p]
    }

    /// Sets a horizontal edge above the point.
    #[inline]
    pub fn set_edge_h(&mut self, p: Point, edge: Option<Edge>) {
        self.edge_h[p] = edge;
    }

    /// Gets a vertical edge on the right of the point.
    #[inline]
    pub fn edge_v(&self, p: Point) -> Option<Edge> {
        self.edge_v[p]
    }

    /// Sets a vertical edge on the right of the point.
    #[inline]
    pub fn set_edge_v(&mut self, p: Point, edge: Option<Edge>) {
        self.edge_v[p] = edge;
    }
}

impl Geom for Puzzle {
    #[inline]
    fn size(&self) -> Size {
        self.size
    }
}

/// An error type which is returned from parsing a string into puzzle.
#[derive(Copy, Clone, Debug)]
pub struct ParsePuzzleError {
    kind: PuzzleErrorKind,
}

/// Puzzle parse result.
pub type ParsePuzzleResult<T> = Result<T, ParsePuzzleError>;

#[derive(Copy, Clone, Debug)]
enum PuzzleErrorKind {
    Empty,
    TooSmallRows,
    TooSmallColumns,
    LengthMismatch,
    InvalidHint,
    Lattice(ParseLatticeError),
}

impl From<ParseLatticeError> for ParsePuzzleError {
    fn from(err: ParseLatticeError) -> ParsePuzzleError {
        ParsePuzzleError { kind: PuzzleErrorKind::Lattice(err) }
    }
}

impl Error for ParsePuzzleError {
    fn description(&self) -> &str {
        use self::PuzzleErrorKind::*;
        match self.kind {
            Empty => "cannot parse puzzle from empty string",
            TooSmallRows => "the number of rows is too small to parse puzzle",
            TooSmallColumns => "the number of columns is too small to parse puzzle",
            LengthMismatch => "the length of lines are not same",
            InvalidHint => "invalid hint found in string",
            Lattice(ref e) => e.description(),
        }
    }
    fn cause(&self) -> Option<&Error> {
        use self::PuzzleErrorKind::*;
        match self.kind {
            Empty | TooSmallRows | TooSmallColumns | LengthMismatch | InvalidHint => None,
            Lattice(ref e) => Some(e),
        }
    }
}

impl fmt::Display for ParsePuzzleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(f)
    }
}

impl ParsePuzzleError {
    fn empty() -> ParsePuzzleError {
        ParsePuzzleError { kind: PuzzleErrorKind::Empty }
    }
    fn too_small_rows() -> ParsePuzzleError {
        ParsePuzzleError { kind: PuzzleErrorKind::TooSmallRows }
    }
    fn too_small_columns() -> ParsePuzzleError {
        ParsePuzzleError { kind: PuzzleErrorKind::TooSmallColumns }
    }
    fn length_mismatch() -> ParsePuzzleError {
        ParsePuzzleError { kind: PuzzleErrorKind::LengthMismatch }
    }
    fn invalid_hint() -> ParsePuzzleError {
        ParsePuzzleError { kind: PuzzleErrorKind::InvalidHint }
    }
}

mod from_str_impl {
    use super::{Puzzle, Edge, ParsePuzzleError as Error};
    use std::str::FromStr;
    use geom::Size;
    use lattice_parser::LatticeParser;

    impl FromStr for Puzzle {
        type Err = Error;

        fn from_str(s: &str) -> Result<Puzzle, Error> {
            let mut mat = s.lines()
                           .map(|l| l.trim_matches('\n'))
                           .map(|l| l.chars().collect::<Vec<_>>())
                           .skip_while(|l| l.is_empty())
                           .collect::<Vec<_>>();

            // Drop trailing empty lines
            while mat.last().map(|l| l.len()) == Some(0) {
                let _ = mat.pop();
            }

            if mat.len() == 0 {
                return Err(Error::empty());
            }

            if mat[0].iter().any(|&c| c == '+') {
                parse_pat1(mat)
            } else {
                parse_pat2(mat)
            }
        }
    }

    fn parse_pat1(mat: Vec<Vec<char>>) -> Result<Puzzle, Error> {
        let parser = try!(LatticeParser::from_lines(&mat));

        let rows = parser.num_rows();
        let cols = parser.num_cols();

        if rows <= 1 {
            return Err(Error::too_small_rows());
        }
        if cols <= 1 {
            return Err(Error::too_small_columns());
        }

        let edge_v = parser.v_edges()
                           .map(|(_, s)| {
                               if s.is_empty() {
                                   None
                               } else if s.chars().all(|c| c == 'x') {
                                   Some(Edge::Cross)
                               } else if s.chars().all(|c| c == '|') {
                                   Some(Edge::Line)
                               } else {
                                   None
                               }
                           })
                           .collect();

        let edge_h = parser.h_edges()
                           .map(|(_, s)| {
                               if s.is_empty() {
                                   None
                               } else if s.chars().all(|c| c == 'x') {
                                   Some(Edge::Cross)
                               } else if s.chars().all(|c| c == '-') {
                                   Some(Edge::Line)
                               } else {
                                   None
                               }
                           })
                           .collect();

        let hint = parser.cells()
                         .filter_map(|(_, s)| {
                             match s.trim_matches(' ') {
                                 "0" => Some(Some(0)),
                                 "1" => Some(Some(1)),
                                 "2" => Some(Some(2)),
                                 "3" => Some(Some(3)),
                                 "4" => Some(Some(4)),
                                 "" | "_" | "-" => Some(None),
                                 _ => None,
                             }
                         })
                         .collect::<Vec<_>>();
        if hint.len() != (rows - 1) * (cols - 1) {
            return Err(Error::invalid_hint());
        }

        let size = Size((rows - 1) as i32, (cols - 1) as i32);
        let side = vec![None; (rows - 1) * (cols - 1)];
        Ok(Puzzle::with_data(size, hint, side, edge_v, edge_h))
    }

    fn parse_pat2(mat: Vec<Vec<char>>) -> Result<Puzzle, Error> {
        let row = mat.len();
        assert!(row > 0);
        let col = mat[0].len();
        assert!(col > 0);
        if mat[1..].iter().any(|r| r.len() != col) {
            return Err(Error::length_mismatch());
        }

        let hint = mat.iter()
                      .flat_map(|line| {
                          line.iter().filter_map(|&c| {
                              match c {
                                  '0' => Some(Some(0)),
                                  '1' => Some(Some(1)),
                                  '2' => Some(Some(2)),
                                  '3' => Some(Some(3)),
                                  '4' => Some(Some(4)),
                                  '_' | '-' => Some(None),
                                  _ => None,
                              }
                          })
                      })
                      .collect::<Vec<_>>();
        if hint.len() != row * col {
            return Err(Error::invalid_hint());
        }

        let size = Size(row as i32, col as i32);
        let side = vec![None; row * col];
        let edge_v = vec![None; row * (col + 1)];
        let edge_h = vec![None; (row + 1) * col];
        Ok(Puzzle::with_data(size, hint, side, edge_v, edge_h))
    }
}

mod display_impl {
    use super::{Puzzle, Edge};
    use std::fmt;
    use geom::{Geom, Point};

    struct Cross;
    impl fmt::Display for Cross {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "+")
        }
    }

    struct HEdge<'a>(&'a Puzzle, Point);
    impl<'a> fmt::Display for HEdge<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let HEdge(puzzle, p) = *self;
            match puzzle.edge_h[p] {
                Some(Edge::Cross) => try!(write!(f, "x")),
                Some(Edge::Line) => try!(write!(f, "-")),
                None => try!(write!(f, " ")),
            }
            Ok(())
        }
    }

    struct VEdge<'a>(&'a Puzzle, Point);
    impl<'a> fmt::Display for VEdge<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let VEdge(puzzle, p) = *self;
            match puzzle.edge_v[p] {
                Some(Edge::Cross) => try!(write!(f, "x")),
                Some(Edge::Line) => try!(write!(f, "|")),
                None => try!(write!(f, " ")),
            }
            Ok(())
        }
    }

    struct EdgeRow<'a>(&'a Puzzle, i32);
    impl<'a> fmt::Display for EdgeRow<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let EdgeRow(puzzle, r) = *self;
            for c in 0..puzzle.column() {
                let p = Point(r, c);
                try!(write!(f, "{}", Cross));
                try!(write!(f, "{}", HEdge(puzzle, p)));
            }
            try!(write!(f, "{}", Cross));
            Ok(())
        }
    }

    struct CellRow<'a>(&'a Puzzle, i32);
    impl<'a> fmt::Display for CellRow<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let CellRow(puzzle, r) = *self;
            for c in 0..puzzle.column() {
                let p = Point(r, c);
                try!(write!(f, "{}", VEdge(puzzle, p)));
                match puzzle.hint[p] {
                    Some(n) => try!(write!(f, "{}", n)),
                    None => try!(write!(f, " ")),
                }
            }
            try!(write!(f, "{}", VEdge(puzzle, Point(r, puzzle.column()))));
            Ok(())
        }
    }

    impl fmt::Display for Puzzle {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            for r in 0..self.row() {
                try!(writeln!(f, "{}", EdgeRow(self, r)));
                try!(writeln!(f, "{}", CellRow(self, r)));
            }
            try!(writeln!(f, "{}", EdgeRow(self, self.row())));
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt;
    use std::error::Error;
    use super::{Puzzle, ParsePuzzleError, ParsePuzzleResult};
    use geom::{Geom, Size, Point};

    fn check_error<T>(result: ParsePuzzleResult<T>, error: ParsePuzzleError)
        where T: fmt::Debug
    {
        assert_eq!(result.unwrap_err().description(), error.description());
    }

    #[test]
    fn parse_pattern2() {
        let input = "123___
______
3_____
";
        let puzzle = input.parse::<Puzzle>().unwrap();
        assert_eq!(Size(3, 6), puzzle.size());
        assert_eq!(Some(1), puzzle.hint(Point(0, 0)));
        assert_eq!(Some(2), puzzle.hint(Point(0, 1)));
        assert_eq!(Some(3), puzzle.hint(Point(0, 2)));
        assert_eq!(None, puzzle.hint(Point(0, 3)));
        assert_eq!(None, puzzle.hint(Point(0, 4)));
        assert_eq!(None, puzzle.hint(Point(0, 5)));
        assert_eq!(None, puzzle.hint(Point(1, 0)));
        assert_eq!(None, puzzle.hint(Point(1, 1)));
        assert_eq!(None, puzzle.hint(Point(1, 2)));
        assert_eq!(None, puzzle.hint(Point(1, 3)));
        assert_eq!(None, puzzle.hint(Point(1, 4)));
        assert_eq!(None, puzzle.hint(Point(1, 5)));
        assert_eq!(Some(3), puzzle.hint(Point(2, 0)));
        assert_eq!(None, puzzle.hint(Point(2, 1)));
        assert_eq!(None, puzzle.hint(Point(2, 2)));
        assert_eq!(None, puzzle.hint(Point(2, 3)));
        assert_eq!(None, puzzle.hint(Point(2, 4)));
        assert_eq!(None, puzzle.hint(Point(2, 5)));
        assert_eq!(&puzzle,
                   puzzle.to_string().parse::<Puzzle>().as_ref().unwrap());
    }

    #[test]
    fn parse_pattern2_numonly() {
        let puzzle = "1243".parse::<Puzzle>().unwrap();
        assert_eq!(Size(1, 4), puzzle.size());
        assert_eq!(Some(1), puzzle.hint(Point(0, 0)));
        assert_eq!(Some(2), puzzle.hint(Point(0, 1)));
        assert_eq!(Some(4), puzzle.hint(Point(0, 2)));
        assert_eq!(Some(3), puzzle.hint(Point(0, 3)));
    }

    #[test]
    fn parse_pattern1() {
        let input = "
+--+ +-+!!+asdf
+  + + +xx+
|  |1|    x
|  | |  2 x
+  + + +  +
";
        let output = "+-+ +-+ +
         
+ + + +x+
| |1|  2x
+ + + + +
";

        let puzzle = input.parse::<Puzzle>().unwrap();
        assert_eq!(Some(1), puzzle.hint(Point(1, 1)));
        assert_eq!(Some(2), puzzle.hint(Point(1, 3)));
        assert_eq!(output, puzzle.to_string());
    }

    #[test]
    fn parse_pattern1_noedge() {
        let input = "
+ + + +
 1 2 3
+ + + +
";
        let puzzle = input.parse::<Puzzle>().unwrap();
        assert_eq!(Some(1), puzzle.hint(Point(0, 0)));
        assert_eq!(Some(2), puzzle.hint(Point(0, 1)));
        assert_eq!(Some(3), puzzle.hint(Point(0, 2)));
    }

    #[test]
    fn parse_empty() {
        check_error("".parse::<Puzzle>(), ParsePuzzleError::empty());
        check_error("\n".parse::<Puzzle>(), ParsePuzzleError::empty());
        check_error("\n\n".parse::<Puzzle>(), ParsePuzzleError::empty());
    }
    #[test]
    fn parse_space_only() {
        check_error("  ".parse::<Puzzle>(), ParsePuzzleError::invalid_hint());
        check_error("   \n   ".parse::<Puzzle>(),
                    ParsePuzzleError::invalid_hint());
    }
    #[test]
    fn parse_pattern1_too_small() {
        check_error("+\n+".parse::<Puzzle>(),
                    ParsePuzzleError::too_small_columns());
        check_error("++".parse::<Puzzle>(), ParsePuzzleError::too_small_rows());
    }
    #[test]
    fn parse_invalid_num() {
        check_error("+ + + +\n 5 0 0 0\n+ + + +".parse::<Puzzle>(),
                    ParsePuzzleError::invalid_hint());
        check_error("1253".parse::<Puzzle>(), ParsePuzzleError::invalid_hint());
    }
    #[test]
    fn parse_pattern2_length_mismatch() {
        check_error("1111\n222".parse::<Puzzle>(),
                    ParsePuzzleError::length_mismatch());
    }
}
