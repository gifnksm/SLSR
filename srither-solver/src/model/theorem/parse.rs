// Copyright (c) 2016 srither-solver developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::fmt;
use std::error::Error as ErrorTrait;
use std::str::FromStr;

use srither_core::lattice_parser::{LatticeParser, ParseLatticeError};

use srither_core::geom::{Point, Move, Size};
use model::pattern::{EdgePattern, HintPattern};
use model::theorem::Theorem;

#[derive(Copy, Clone, Debug)]
pub struct ParseTheoremError {
    kind: ParseTheoremErrorKind,
}

#[derive(Copy, Clone, Debug)]
enum ParseTheoremErrorKind {
    NoSeparator,
    TooSmallRows,
    TooSmallColumns,
    SizeMismatch,
    MatcherDisappear,
    Lattice(ParseLatticeError),
}

impl From<ParseLatticeError> for ParseTheoremError {
    fn from(err: ParseLatticeError) -> ParseTheoremError {
        ParseTheoremError { kind: ParseTheoremErrorKind::Lattice(err) }
    }
}

impl ErrorTrait for ParseTheoremError {
    fn description(&self) -> &str {
        use self::ParseTheoremErrorKind::*;
        match self.kind {
            NoSeparator => "cannot found separator `!` in string",
            TooSmallRows => "the number of rows is too small to parse puzzle",
            TooSmallColumns => "the number of columns is too small to parse puzzle",
            SizeMismatch => "size of the matcher does not match size of the pattern",
            MatcherDisappear => "some elements in the matcher disappear in the pattern",
            Lattice(ref e) => e.description(),
        }
    }
    fn cause(&self) -> Option<&ErrorTrait> {
        use self::ParseTheoremErrorKind::*;
        match self.kind {
            NoSeparator | TooSmallRows | TooSmallColumns | SizeMismatch | MatcherDisappear => None,
            Lattice(ref e) => Some(e),
        }
    }
}

impl fmt::Display for ParseTheoremError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(f)
    }
}

impl ParseTheoremError {
    fn no_separator() -> ParseTheoremError {
        ParseTheoremError { kind: ParseTheoremErrorKind::NoSeparator }
    }
    fn too_small_rows() -> ParseTheoremError {
        ParseTheoremError { kind: ParseTheoremErrorKind::TooSmallRows }
    }
    fn too_small_columns() -> ParseTheoremError {
        ParseTheoremError { kind: ParseTheoremErrorKind::TooSmallColumns }
    }
    fn size_mismatch() -> ParseTheoremError {
        ParseTheoremError { kind: ParseTheoremErrorKind::SizeMismatch }
    }
    fn matcher_disappear() -> ParseTheoremError {
        ParseTheoremError { kind: ParseTheoremErrorKind::MatcherDisappear }
    }
}

impl FromStr for Theorem {
    type Err = ParseTheoremError;

    fn from_str(s: &str) -> Result<Theorem, ParseTheoremError> {
        use self::ParseTheoremError as Error;

        let mut matcher_lines = vec![];
        let mut result_lines = vec![];
        let mut closed_lines = vec![];

        let lines = s.lines()
                     .map(|l| l.trim_matches('\n'))
                     .filter(|s| !s.is_empty());

        for line in lines {
            let mut it = line.splitn(3, '!');

            matcher_lines.push(it.next().unwrap().chars().collect());

            if let Some(l) = it.next() {
                result_lines.push(l.chars().collect());
            } else {
                return Err(Error::no_separator());
            }

            if let Some(l) = it.next() {
                closed_lines.push(l.chars().collect());
            }
        }

        let (m_size, m_hint_pat, m_edge_pat) = try!(parse_lines(&matcher_lines));
        let (r_size, r_hint_pat, mut r_edge_pat) = try!(parse_lines(&result_lines));
        if m_size != r_size {
            return Err(Error::size_mismatch());
        }

        let c_hint_pat = if closed_lines.is_empty() {
            None
        } else {
            let (c_size, c_hint_pat, _c_edge_pat) = try!(parse_lines(&closed_lines));
            if m_size != c_size {
                return Err(Error::size_mismatch());
            }
            Some(c_hint_pat)
        };

        if m_hint_pat != r_hint_pat {
            return Err(Error::matcher_disappear());
        }

        let mut idx = 0;
        for &p in &m_edge_pat {
            match r_edge_pat[idx..].iter().position(|&x| x == p) {
                Some(i) => {
                    idx += i;
                    let _ = r_edge_pat.remove(idx);
                }
                None => {
                    return Err(Error::matcher_disappear());
                }
            }
        }

        let c_pat = c_hint_pat.map(|pat| {
            use std::ops::Add;
            let sum = pat.iter().map(|h| h.hint() as u32).fold(0, Add::add);
            (sum, pat)
        });

        return Ok(Theorem {
            size: m_size,
            hint_matcher: m_hint_pat,
            edge_matcher: m_edge_pat,
            result: r_edge_pat,
            closed_hint: c_pat,
        });

        fn parse_lines
            (lines: &[Vec<char>])
             -> Result<(Size, Vec<HintPattern>, Vec<EdgePattern<Point>>), ParseTheoremError> {
            let parser = try!(LatticeParser::from_lines(lines));

            let rows = parser.num_rows();
            let cols = parser.num_cols();

            if rows <= 1 {
                return Err(Error::too_small_rows());
            }
            if cols <= 1 {
                return Err(Error::too_small_columns());
            }

            let size = Size((rows - 1) as i32, (cols - 1) as i32);

            let mut hint_pat = vec![];
            let mut edge_pat = vec![];

            for (p, s) in parser.v_edges() {
                if s.is_empty() {
                    continue;
                }
                if s.chars().all(|c| c == 'x') {
                    edge_pat.push(EdgePattern::cross(p + Move::LEFT, p));
                    continue;
                }
                if s.chars().all(|c| c == '|') {
                    edge_pat.push(EdgePattern::line(p + Move::LEFT, p));
                    continue;
                }
            }

            for (p, s) in parser.h_edges() {
                if s.is_empty() {
                    continue;
                }
                if s.chars().all(|c| c == 'x') {
                    edge_pat.push(EdgePattern::cross(p + Move::UP, p));
                    continue;
                }
                if s.chars().all(|c| c == '-') {
                    edge_pat.push(EdgePattern::line(p + Move::UP, p));
                    continue;
                }
            }

            let mut pairs: Vec<(char, Vec<Point>, Vec<Point>)> = vec![];

            for (p, s) in parser.cells() {
                for c in s.trim_matches(' ').chars() {
                    match c {
                        '0' => {
                            hint_pat.push(HintPattern::new(0, p));
                        }
                        '1' => {
                            hint_pat.push(HintPattern::new(1, p));
                        }
                        '2' => {
                            hint_pat.push(HintPattern::new(2, p));
                        }
                        '3' => {
                            hint_pat.push(HintPattern::new(3, p));
                        }
                        '4' => {
                            hint_pat.push(HintPattern::new(4, p));
                        }
                        _ if c.is_alphabetic() => {
                            let key = c.to_lowercase().next().unwrap();
                            match pairs.iter().position(|&(k, _, _)| k == key) {
                                Some(idx) => {
                                    if c.is_lowercase() {
                                        pairs[idx].1.push(p);
                                    } else {
                                        pairs[idx].2.push(p);
                                    }
                                }
                                None => {
                                    let (lower, upper) = if c.is_lowercase() {
                                        (vec![p], vec![])
                                    } else {
                                        (vec![], vec![p])
                                    };
                                    pairs.push((key, lower, upper));
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            for &(_, ref ps0, ref ps1) in &pairs {
                if !ps0.is_empty() && !ps1.is_empty() {
                    edge_pat.push(EdgePattern::line(ps0[0], ps1[0]));
                }

                if ps0.len() > 0 {
                    for &p in &ps0[1..] {
                        edge_pat.push(EdgePattern::cross(ps0[0], p));
                    }
                }
                if ps1.len() > 0 {
                    for &p in &ps1[1..] {
                        edge_pat.push(EdgePattern::cross(ps1[0], p));
                    }
                }
            }

            hint_pat.sort();
            hint_pat.dedup();
            edge_pat.sort();
            edge_pat.dedup();
            Ok((size, hint_pat, edge_pat))
        }
    }
}


#[cfg(test)]
mod tests {
    use srither_core::geom::{Point, Size};
    use model::pattern::{EdgePattern, HintPattern};
    use model::theorem::Theorem;

    #[test]
    fn parse() {
        fn check(size: Size,
                 mut hint_matcher: Vec<HintPattern>,
                 mut edge_matcher: Vec<EdgePattern<Point>>,
                 mut result: Vec<EdgePattern<Point>>,
                 input: &str) {
            hint_matcher.sort();
            hint_matcher.dedup();
            edge_matcher.sort();
            edge_matcher.dedup();
            result.sort();
            result.dedup();
            let theo = Theorem {
                size: size,
                hint_matcher: hint_matcher,
                edge_matcher: edge_matcher,
                result: result,
                closed_hint: None,
            };
            assert_eq!(theo, input.parse::<Theorem>().unwrap())
        }

        check(Size(1, 1),
              vec![HintPattern::new(0, Point(0, 0))],
              vec![],
              vec![EdgePattern::cross(Point(0, 0), Point(0, -1)),
                   EdgePattern::cross(Point(0, 0), Point(0, 1)),
                   EdgePattern::cross(Point(0, 0), Point(-1, 0)),
                   EdgePattern::cross(Point(0, 0), Point(1, 0))],
              r"
+ + ! +x+
 0  ! x0x
+ + ! +x+
");
        check(Size(3, 3),
              vec![HintPattern::new(0, Point(1, 0)), HintPattern::new(3, Point(1, 1))],
              vec![],
              vec![EdgePattern::cross(Point(1, 0), Point(1, -1)),
                   EdgePattern::cross(Point(1, 0), Point(1, 1)),
                   EdgePattern::cross(Point(1, 0), Point(0, 0)),
                   EdgePattern::cross(Point(1, 0), Point(2, 0)),
                   EdgePattern::cross(Point(0, 1), Point(0, 2)),
                   EdgePattern::cross(Point(1, 2), Point(0, 2)),
                   EdgePattern::cross(Point(1, 2), Point(2, 2)),
                   EdgePattern::cross(Point(2, 1), Point(2, 2)),
                   EdgePattern::line(Point(0, 0), Point(0, 1)),
                   EdgePattern::line(Point(0, 1), Point(1, 1)),
                   EdgePattern::line(Point(1, 1), Point(1, 2)),
                   EdgePattern::line(Point(1, 1), Point(2, 1)),
                   EdgePattern::line(Point(2, 0), Point(2, 1))],
              r"
+ + + + ! + + + +
        !   | x
+ + + + ! +x+-+x+
 0 3    ! x0x3|
+ + + + ! +x+-+x+
        !   | x
+ + + + ! + + + +
");
        check(Size(2, 2),
              vec![HintPattern::new(1, Point(1, 1))],
              vec![EdgePattern::line(Point(1, 0), Point(0, 1))],
              vec![EdgePattern::cross(Point(1, 1), Point(1, 2)),
                   EdgePattern::cross(Point(1, 1), Point(2, 1))],
              r"
+ + + ! + + +
   a  !    a
+ + + ! + + +
 A 1  !  A 1x
+ + + ! + +x+
");
        check(Size(3, 3),
              vec![HintPattern::new(3, Point(1, 1))],
              vec![EdgePattern::cross(Point(1, 0), Point(0, 1))],
              vec![EdgePattern::cross(Point(0, 0), Point(0, 1)),
                   EdgePattern::cross(Point(0, 0), Point(1, 0)),
                   EdgePattern::line(Point(0, 1), Point(1, 1)),
                   EdgePattern::line(Point(1, 0), Point(1, 1)),
                   EdgePattern::line(Point(1, 2), Point(2, 1))],
              r"
+ + + + ! + + + +
   a    !   xa
+ + + + ! +x+-+ +
 a 3    !  a|3 b
+ + + + ! + + + +
        !    B
+ + + + ! + + + +
");
    }
}
