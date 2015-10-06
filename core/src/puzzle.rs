use std::error::Error;
use std::fmt;

use ::geom::{Geom, Size, Table};
use ::lattice_parser::ParseLatticeError;

pub type Hint = Option<u8>;
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Side { In, Out }
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Edge { Line, Cross }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Puzzle {
    size: Size,
    hint: Table<Hint>,
    side: Table<Option<Side>>,
    edge_v: Table<Option<Edge>>,
    edge_h: Table<Option<Edge>>,
    sum_of_hint: Option<u32>
}

impl Puzzle {
    #[inline]
    pub fn new(size: Size) -> Puzzle {
        assert!(size.0 > 0 && size.1 > 0);
        let hint = Table::new_empty(size, None, None);
        let side = Table::new_empty(size, Some(Side::Out), None);
        let edge_v = Table::new_empty(Size(size.0, size.1 + 1), Some(Edge::Cross), None);
        let edge_h = Table::new_empty(Size(size.0 + 1, size.1), Some(Edge::Cross), None);
        Puzzle {
            size: size, hint: hint, side: side, edge_v: edge_v, edge_h: edge_h,
            sum_of_hint: None
        }
    }

    #[inline]
    fn with_data(size: Size, hint: Vec<Hint>, side: Vec<Option<Side>>,
                 edge_v: Vec<Option<Edge>>, edge_h: Vec<Option<Edge>>) -> Puzzle {
        assert!(size.0 > 0 && size.1 > 0);
        let hint = Table::new(size, None, hint);
        let side = Table::new(size, Some(Side::Out), side);
        let edge_v = Table::new(Size(size.0, size.1 + 1), Some(Edge::Cross), edge_v);
        let edge_h = Table::new(Size(size.0 + 1, size.1), Some(Edge::Cross), edge_h);
        Puzzle {
            size: size, hint: hint, side: side, edge_v: edge_v, edge_h: edge_h,
            sum_of_hint: None
        }
    }

    #[inline]
    pub fn hint(&self) -> &Table<Hint> { &self.hint }
    #[inline]
    pub fn side(&self) -> &Table<Option<Side>> { &self.side }
    #[inline]
    pub fn edge_h(&self) -> &Table<Option<Edge>> { &self.edge_h }
    #[inline]
    pub fn edge_v(&self) -> &Table<Option<Edge>> { &self.edge_v }

    #[inline]
    pub fn hint_mut(&mut self) -> &mut Table<Hint> {
        self.sum_of_hint = None;
        &mut self.hint
    }
    #[inline]
    pub fn side_mut(&mut self) -> &mut Table<Option<Side>> { &mut self.side }
    #[inline]
    pub fn edge_h_mut(&mut self) -> &mut Table<Option<Edge>> { &mut self.edge_h }
    #[inline]
    pub fn edge_v_mut(&mut self) -> &mut Table<Option<Edge>> { &mut self.edge_v }
}

impl Geom for Puzzle {
    #[inline]
    fn size(&self) -> Size { self.size }
}

#[derive(Copy, Clone, Debug)]
pub struct ParsePuzzleError {
    kind: PuzzleErrorKind
}

#[derive(Copy, Clone, Debug)]
enum PuzzleErrorKind {
    Empty,
    TooSmallRows,
    TooSmallColumns,
    LengthMismatch,
    InvalidHint,
    Lattice(ParseLatticeError)
}

impl From<ParseLatticeError> for ParsePuzzleError {
    fn from(err: ParseLatticeError) -> ParsePuzzleError {
        ParsePuzzleError {
            kind: PuzzleErrorKind::Lattice(err)
        }
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
            Lattice(ref e) => e.description()
        }
    }
    fn cause(&self) -> Option<&Error> {
        use self::PuzzleErrorKind::*;
        match self.kind {
            Empty | TooSmallRows | TooSmallColumns | LengthMismatch | InvalidHint
                => None,
            Lattice(ref e) => Some(e)
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

            if mat.len() == 0 { return Err(Error::empty()) }

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

        if rows <= 1 { return Err(Error::too_small_rows()) }
        if cols <= 1 { return Err(Error::too_small_columns()) }

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
            }).collect();

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
            }).collect();

        let hint = parser.cells()
            .filter_map(|(_, s)| {
                match s.trim_matches(' ') {
                    "0" => Some(Some(0)),
                    "1" => Some(Some(1)),
                    "2" => Some(Some(2)),
                    "3" => Some(Some(3)),
                    "4" => Some(Some(4)),
                    "" | "_" | "-" => Some(None),
                    _ => None
                }
            }).collect::<Vec<_>>();
        if hint.len() != (rows - 1) * (cols - 1) {
            return Err(Error::invalid_hint())
        }

        let size = Size((rows - 1) as i32, (cols - 1) as i32);
        let side = vec![None; (rows - 1) * (cols - 1)];
        Ok(Puzzle::with_data(size, hint, side, edge_v, edge_h))
    }

    fn parse_pat2(mat: Vec<Vec<char>>) -> Result<Puzzle, Error> {
        let row = mat.len();
        if row < 1 { return Err(Error::too_small_rows()) }
        let col = mat[0].len();
        if col < 1 { return Err(Error::too_small_columns()) }
        if mat[1..].iter().any(|r| r.len() != col) {
            return Err(Error::length_mismatch())
        }

        let hint = mat.iter().flat_map(|line| {
            line.iter().filter_map(|&c| {
                match c {
                    '0' => Some(Some(0)),
                    '1' => Some(Some(1)),
                    '2' => Some(Some(2)),
                    '3' => Some(Some(3)),
                    '4' => Some(Some(4)),
                    '_' | '-' => Some(None),
                    _ => None
                }
            })
        }).collect::<Vec<_>>();
        if hint.len() != row * col {
            return Err(Error::invalid_hint())
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
                None => try!(write!(f, " "))
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
                None => try!(write!(f, " "))
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
                    None => try!(write!(f, " "))
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
    use super::Puzzle;
    use geom::{Geom, Size, Point};

    #[test]
    fn parse() {
        let input = "123___
______
3_____
";
        let puzzle = input.parse::<Puzzle>().unwrap();
        let hint = puzzle.hint();
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

        assert_eq!(&puzzle,
                   puzzle.to_string().parse::<Puzzle>().as_ref().unwrap());

        let puzzle = "1243".parse::<Puzzle>().unwrap();
        let hint = puzzle.hint();
        assert_eq!(Size(1, 4), hint.size());
        assert_eq!(Some(1), hint[Point(0, 0)]);
        assert_eq!(Some(2), hint[Point(0, 1)]);
        assert_eq!(Some(4), hint[Point(0, 2)]);
        assert_eq!(Some(3), hint[Point(0, 3)]);

        assert!("1253".parse::<Puzzle>().is_err());

        let input = "
+--+ +-+!!+asdf
+  + + +  +
|  |1|    |
|  | |  2 |
+  + + +  +
";
        let output = "+-+ +-+ +
         
+ + + + +
| |1|  2|
+ + + + +
";

        let puzzle = input.parse::<Puzzle>().unwrap();
        let hint = puzzle.hint();
        assert_eq!(Some(1), hint[Point(1, 1)]);
        assert_eq!(Some(2), hint[Point(1, 3)]);
        assert_eq!(output, puzzle.to_string());

        assert!("".parse::<Puzzle>().is_err());

        let input = "
+ + + +
 1 2 3
+ + + +
";
        let puzzle = input.parse::<Puzzle>().unwrap();
        let hint = puzzle.hint();
        assert_eq!(Some(1), hint[Point(0, 0)]);
        assert_eq!(Some(2), hint[Point(0, 1)]);
        assert_eq!(Some(3), hint[Point(0, 2)]);
    }
}
