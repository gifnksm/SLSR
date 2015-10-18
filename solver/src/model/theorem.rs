use std::fmt;
use std::slice::Iter as SliceIter;
use std::str::FromStr;
use std::error::Error as ErrorTrait;

use slsr_core::puzzle::{Edge, Puzzle};
use slsr_core::geom::{CellId, Geom, Point, Rotation, Move, Size};
use slsr_core::lattice_parser::{LatticeParser, ParseLatticeError};

use {Error, State, SolverResult};
use model::side_map::SideMap;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct HintPattern {
    hint: u8,
    point: Point,
}

impl HintPattern {
    fn new(h: u8, p: Point) -> HintPattern {
        HintPattern {
            hint: h,
            point: p,
        }.normalized()
    }
    pub fn hint(&self) -> u8 {
        self.hint
    }
    pub fn point(&self) -> Point {
        self.point
    }

    fn normalized(self) -> HintPattern {
        self
    }
    fn rotate(mut self, rot: Rotation) -> HintPattern {
        let o = Point(0, 0);
        let p = self.point;
        self.point = o + rot * (p - o);
        self.normalized()
    }
    fn shift(mut self, d: Move) -> HintPattern {
        let p = self.point;
        self.point = p + d;
        self.normalized()
    }

    fn matches<T>(self, puzzle: &Puzzle) -> SolverResult<PatternMatchResult<T>> {
        if puzzle.hint(self.point) == Some(self.hint) {
            Ok(PatternMatchResult::Complete)
        } else {
            Ok(PatternMatchResult::Conflict)
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct EdgePattern<P> {
    edge: Edge,
    points: (P, P),
}

impl EdgePattern<Point> {
    fn cross(p0: Point, p1: Point) -> EdgePattern<Point> {
        EdgePattern {
            edge: Edge::Cross,
            points: (p0, p1),
        }
            .normalized()
    }
    fn line(p0: Point, p1: Point) -> EdgePattern<Point> {
        EdgePattern {
            edge: Edge::Line,
            points: (p0, p1),
        }
            .normalized()
    }

    fn normalized(self) -> EdgePattern<Point> {
        let mut points = self.points;
        if self.points.1 < self.points.0 {
            points = (self.points.1, self.points.0);
        }
        EdgePattern {
            edge: self.edge,
            points: points,
        }
    }
    fn rotate(mut self, rot: Rotation) -> EdgePattern<Point> {
        let o = Point(0, 0);
        let ps = self.points;
        self.points = (o + rot * (ps.0 - o), o + rot * (ps.1 - o));
        self.normalized()
    }
    fn shift(mut self, d: Move) -> EdgePattern<Point> {
        let ps = self.points;
        self.points = (ps.0 + d, ps.1 + d);
        self.normalized()
    }

    fn to_cellid(self, size: Size) -> EdgePattern<CellId> {
        let p0 = size.point_to_cellid(self.points.0);
        let p1 = size.point_to_cellid(self.points.1);
        EdgePattern {
            edge: self.edge,
            points: (p0, p1),
        }
    }

    fn matches(self,
               size: Size,
               side_map: &mut SideMap)
               -> SolverResult<PatternMatchResult<EdgePattern<CellId>>> {
        self.to_cellid(size).matches(side_map)
    }
}

impl EdgePattern<CellId> {
    fn matches(self,
               side_map: &mut SideMap)
               -> SolverResult<PatternMatchResult<EdgePattern<CellId>>> {
        let ps = self.points;
        match side_map.get_edge(ps.0, ps.1) {
            State::Fixed(edg) => {
                if self.edge == edg {
                    Ok(PatternMatchResult::Complete)
                } else {
                    Ok(PatternMatchResult::Conflict)
                }
            }
            State::Unknown => Ok(PatternMatchResult::Partial(self)),
            State::Conflict => Err(Error::invalid_board()),
        }
    }

    pub fn apply(&self, side_map: &mut SideMap) {
        let ps = self.points;
        let _ = side_map.set_edge(ps.0, ps.1, self.edge);
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Pattern {
    Hint(HintPattern),
    Edge(EdgePattern<Point>),
}

enum PatternMatchResult<T> {
    Complete,
    Partial(T),
    Conflict,
}

impl Pattern {
    fn hint(h: u8, p: Point) -> Pattern {
        Pattern::Hint(HintPattern::new(h, p))
    }
    fn cross(p0: Point, p1: Point) -> Pattern {
        Pattern::Edge(EdgePattern::cross(p0, p1))
    }
    fn line(p0: Point, p1: Point) -> Pattern {
        Pattern::Edge(EdgePattern::line(p0, p1))
    }

    fn rotate(self, rot: Rotation) -> Pattern {
        match self {
            Pattern::Hint(h) => Pattern::Hint(h.rotate(rot)),
            Pattern::Edge(e) => Pattern::Edge(e.rotate(rot)),
        }
    }
    fn shift(self, d: Move) -> Pattern {
        match self {
            Pattern::Hint(h) => Pattern::Hint(h.shift(d)),
            Pattern::Edge(e) => Pattern::Edge(e.shift(d)),
        }
    }

    fn matches(self,
               puzzle: &Puzzle,
               side_map: &mut SideMap)
               -> SolverResult<PatternMatchResult<EdgePattern<CellId>>> {
        match self {
            Pattern::Hint(h) => h.matches(puzzle),
            Pattern::Edge(e) => e.matches(puzzle.size(), side_map),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Theorem {
    size: Size,
    matcher: Vec<Pattern>,
    result: Vec<EdgePattern<Point>>,
    closed_hint: Option<(u32, Vec<HintPattern>)>,
}

impl Theorem {
    fn normalized(mut self) -> Theorem {
        self.matcher.sort();
        self.matcher.dedup();
        self.result.sort();
        self.result.dedup();
        if let Some((_, ref mut closed)) = self.closed_hint {
            closed.sort();
            closed.dedup();
        }
        self
    }

    fn rotate(mut self, rot: Rotation) -> Theorem {
        let size = rot * Move(self.size.0, self.size.1);

        let mut d = Move(0, 0);
        if size.0 < 0 {
            d = d + Move(-size.0 - 1, 0);
        }
        if size.1 < 0 {
            d = d + Move(0, -size.1 - 1);
        }

        self.size = Size(size.0.abs(), size.1.abs());
        for x in self.matcher.iter_mut() {
            *x = x.rotate(rot).shift(d);
        }
        for x in self.result.iter_mut() {
            *x = x.rotate(rot).shift(d)
        }
        if let Some((_, ref mut closed)) = self.closed_hint {
            for x in closed.iter_mut() {
                *x = x.rotate(rot).shift(d);
            }
        }

        self.normalized()
    }

    pub fn shift(mut self, d: Move) -> Theorem {
        for x in self.matcher.iter_mut() {
            *x = x.shift(d);
        }
        for x in self.result.iter_mut() {
            *x = x.shift(d);
        }
        if let Some((_, ref mut closed)) = self.closed_hint {
            for x in closed.iter_mut() {
                *x = x.shift(d);
            }
        }

        self
    }

    pub fn all_rotations(self) -> Vec<Theorem> {
        let deg90 = self.clone().rotate(Rotation::UCW90);
        let deg180 = self.clone().rotate(Rotation::UCW180);
        let deg270 = self.clone().rotate(Rotation::UCW270);
        let h_deg0 = self.clone().rotate(Rotation::H_FLIP);
        let h_deg90 = h_deg0.clone().rotate(Rotation::UCW90);
        let h_deg180 = h_deg0.clone().rotate(Rotation::UCW180);
        let h_deg270 = h_deg0.clone().rotate(Rotation::UCW270);
        let mut rots = vec![self.clone(),
                            deg90,
                            deg180,
                            deg270,
                            h_deg0,
                            h_deg90,
                            h_deg180,
                            h_deg270];

        rots.sort();
        rots.dedup();

        rots
    }

    pub fn size(&self) -> Size {
        self.size
    }
    pub fn head(&self) -> Pattern {
        self.matcher[0]
    }

    fn can_close(puzzle: &Puzzle,
                 sum_of_hint: u32,
                 hpat: &[HintPattern],
                 sum_of_hpat: u32)
                 -> bool {
        if sum_of_hint > sum_of_hpat {
            return false;
        }

        let mut ava_sum = 0;
        for h in hpat {
            if let Some(n) = puzzle.hint(h.point) {
                if n != h.hint {
                    return false;
                }
                ava_sum += n as u32;
            }
        }

        if ava_sum != sum_of_hint {
            return false;
        }

        return true;
    }

    pub fn matches(self,
                   puzzle: &Puzzle,
                   sum_of_hint: u32,
                   side_map: &mut SideMap)
                   -> SolverResult<TheoremMatchResult> {
        let cap = self.matcher.len();
        let mut new_matcher = Vec::with_capacity(cap);

        for matcher in self.matcher {
            match try!(matcher.matches(puzzle, side_map)) {
                PatternMatchResult::Complete => {}
                PatternMatchResult::Partial(m) => new_matcher.push(m),
                PatternMatchResult::Conflict => {
                    return Ok(TheoremMatchResult::Conflict);
                }
            }
        }

        if let Some((sum_of_hpat, ref hpat)) = self.closed_hint {
            if Theorem::can_close(puzzle, sum_of_hint, hpat, sum_of_hpat) {
                return Ok(TheoremMatchResult::Conflict);
            }
        }

        let result = self.result
                         .into_iter()
                         .map(|pat| pat.to_cellid(puzzle.size()))
                         .collect();

        if new_matcher.is_empty() {
            Ok(TheoremMatchResult::Complete(result))
        } else {
            Ok(TheoremMatchResult::Partial(TheoremMatcher {
                matcher: new_matcher,
                result: result,
            }))
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TheoremMatcher {
    matcher: Vec<EdgePattern<CellId>>,
    result: Vec<EdgePattern<CellId>>,
}

#[derive(Clone, Debug)]
pub enum TheoremMatchResult {
    Complete(Vec<EdgePattern<CellId>>),
    Partial(TheoremMatcher),
    Conflict,
}

impl TheoremMatchResult {
    pub fn update(self, side_map: &mut SideMap, new_theorem: &mut Vec<TheoremMatcher>) {
        match self {
            TheoremMatchResult::Complete(result) => {
                for pat in &result {
                    pat.apply(side_map);
                }
            }
            TheoremMatchResult::Partial(theo) => {
                new_theorem.push(theo)
            }
            TheoremMatchResult::Conflict => {}
        }
    }
}

impl TheoremMatcher {
    pub fn merge(&mut self, other: &TheoremMatcher) -> Result<(), ()> {
        if self.matcher != other.matcher {
            return Err(());
        }

        self.result.extend(other.result.iter().cloned());
        self.result.sort();
        self.result.dedup();
        Ok(())
    }

    pub fn matches(mut self, side_map: &mut SideMap) -> SolverResult<TheoremMatchResult> {
        unsafe {
            // Assume the elements of self.matcher is copyable.
            let len = self.matcher.len();
            let p = self.matcher.as_mut_ptr();
            let mut w = 0;
            for r in 0..len {
                let read = *p.offset(r as isize);

                match try!(read.matches(side_map)) {
                    PatternMatchResult::Complete => {}
                    PatternMatchResult::Partial(e) => {
                        *p.offset(w as isize) = e;
                        w += 1;
                    }
                    PatternMatchResult::Conflict => {
                        return Ok(TheoremMatchResult::Conflict);
                    }
                }
            }
            self.matcher.set_len(w);
        }

        let m = if self.matcher.is_empty() {
            TheoremMatchResult::Complete(self.result)
        } else {
            TheoremMatchResult::Partial(self)
        };
        Ok(m)
    }

    pub fn num_matcher(&self) -> usize {
        self.matcher.len()
    }

    pub fn matcher_edges<'a>(&'a self) -> Edges<'a> {
        Edges { iter: self.matcher.iter() }
    }

    pub fn result_edges<'a>(&'a self) -> Edges<'a> {
        Edges { iter: self.result.iter() }
    }
}

pub struct Edges<'a> {
    iter: SliceIter<'a, EdgePattern<CellId>>,
}

impl<'a> Iterator for Edges<'a> {
    type Item = (Edge, (CellId, CellId));

    fn next(&mut self) -> Option<(Edge, (CellId, CellId))> {
        self.iter.next().map(|pat| (pat.edge, pat.points))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ParseTheoremError {
    kind: TheoremErrorKind,
}

#[derive(Copy, Clone, Debug)]
enum TheoremErrorKind {
    NoSeparator,
    TooSmallRows,
    TooSmallColumns,
    SizeMismatch,
    MatcherDisappear,
    Lattice(ParseLatticeError),
}

impl From<ParseLatticeError> for ParseTheoremError {
    fn from(err: ParseLatticeError) -> ParseTheoremError {
        ParseTheoremError { kind: TheoremErrorKind::Lattice(err) }
    }
}

impl ErrorTrait for ParseTheoremError {
    fn description(&self) -> &str {
        use self::TheoremErrorKind::*;
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
        use self::TheoremErrorKind::*;
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
        ParseTheoremError { kind: TheoremErrorKind::NoSeparator }
    }
    fn too_small_rows() -> ParseTheoremError {
        ParseTheoremError { kind: TheoremErrorKind::TooSmallRows }
    }
    fn too_small_columns() -> ParseTheoremError {
        ParseTheoremError { kind: TheoremErrorKind::TooSmallColumns }
    }
    fn size_mismatch() -> ParseTheoremError {
        ParseTheoremError { kind: TheoremErrorKind::SizeMismatch }
    }
    fn matcher_disappear() -> ParseTheoremError {
        ParseTheoremError { kind: TheoremErrorKind::MatcherDisappear }
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

        let (m_size, m_pat) = try!(parse_lines(&matcher_lines));
        let (r_size, mut r_pat) = try!(parse_lines(&result_lines));
        if m_size != r_size {
            return Err(Error::size_mismatch());
        }

        let c_pat = if closed_lines.is_empty() {
            None
        } else {
            let (c_size, c_pat) = try!(parse_lines(&closed_lines));
            if m_size != c_size {
                return Err(Error::size_mismatch());
            }
            Some(c_pat)
        };

        let mut idx = 0;
        for &p in &m_pat {
            match r_pat[idx..].iter().position(|&x| x == p) {
                Some(i) => {
                    idx += i;
                    let _ = r_pat.remove(idx);
                }
                None => {
                    return Err(Error::matcher_disappear());
                }
            }
        }

        let r_pat = r_pat.into_iter()
                         .map(|pat| {
                             match pat {
                                 Pattern::Edge(e) => e,
                                 _ => panic!(),
                             }
                         })
                         .collect();

        let c_pat = c_pat.map(|pats| {
            use std::ops::Add;
            let hints = pats.into_iter()
                            .filter_map(|pat| {
                                match pat {
                                    Pattern::Hint(h) => Some(h),
                                    _ => None,
                                }
                            })
                            .collect::<Vec<_>>();
            let sum = hints.iter().map(|h| h.hint as u32).fold(0, Add::add);
            (sum, hints)
        });

        return Ok(Theorem {
            size: m_size,
            matcher: m_pat,
            result: r_pat,
            closed_hint: c_pat,
        });

        fn parse_lines(lines: &[Vec<char>]) -> Result<(Size, Vec<Pattern>), ParseTheoremError> {
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

            let mut pat = vec![];

            for (p, s) in parser.v_edges() {
                if s.is_empty() {
                    continue;
                }
                if s.chars().all(|c| c == 'x') {
                    pat.push(Pattern::cross(p + Move::LEFT, p));
                    continue;
                }
                if s.chars().all(|c| c == '|') {
                    pat.push(Pattern::line(p + Move::LEFT, p));
                    continue;
                }
            }

            for (p, s) in parser.h_edges() {
                if s.is_empty() {
                    continue;
                }
                if s.chars().all(|c| c == 'x') {
                    pat.push(Pattern::cross(p + Move::UP, p));
                    continue;
                }
                if s.chars().all(|c| c == '-') {
                    pat.push(Pattern::line(p + Move::UP, p));
                    continue;
                }
            }

            let mut pairs: Vec<(char, Vec<Point>, Vec<Point>)> = vec![];

            for (p, s) in parser.cells() {
                for c in s.trim_matches(' ').chars() {
                    match c {
                        '0' => {
                            pat.push(Pattern::hint(0, p));
                        }
                        '1' => {
                            pat.push(Pattern::hint(1, p));
                        }
                        '2' => {
                            pat.push(Pattern::hint(2, p));
                        }
                        '3' => {
                            pat.push(Pattern::hint(3, p));
                        }
                        '4' => {
                            pat.push(Pattern::hint(4, p));
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
                    pat.push(Pattern::line(ps0[0], ps1[0]));
                }

                if ps0.len() > 0 {
                    for &p in &ps0[1..] {
                        pat.push(Pattern::cross(ps0[0], p));
                    }
                }
                if ps1.len() > 0 {
                    for &p in &ps1[1..] {
                        pat.push(Pattern::cross(ps1[0], p));
                    }
                }
            }

            pat.sort();
            pat.dedup();
            Ok((size, pat))
        }
    }
}

#[cfg(test)]
mod tests {
    use slsr_core::geom::{Point, Size, Rotation};
    use super::{EdgePattern, Pattern, Theorem};

    #[test]
    fn parse() {
        fn check(size: Size,
                 mut matcher: Vec<Pattern>,
                 mut result: Vec<EdgePattern<Point>>,
                 input: &str) {
            matcher.sort();
            matcher.dedup();
            result.sort();
            result.dedup();
            let theo = Theorem {
                size: size,
                matcher: matcher,
                result: result,
                closed_hint: None,
            };
            assert_eq!(theo, input.parse::<Theorem>().unwrap())
        }

        check(Size(1, 1),
              vec![Pattern::hint(0, Point(0, 0))],
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
              vec![Pattern::hint(0, Point(1, 0)), Pattern::hint(3, Point(1, 1))],
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
              vec![Pattern::hint(1, Point(1, 1)), Pattern::line(Point(1, 0), Point(0, 1))],
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
              vec![Pattern::hint(3, Point(1, 1)), Pattern::cross(Point(1, 0), Point(0, 1))],
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

    #[test]
    fn rotate() {
        let deg0 = r"
+ + + ! + + +
   a  !  bxa
+ + + ! +x+-+
 a 3  !  a|3
+ + + ! + + +
      !    B
+ + + ! + + +
"
                       .parse::<Theorem>()
                       .unwrap();

        let deg90 = r"
+ + + + ! + + + +
 a 3    !  a|3 B
+ + + + ! +x+-+ +
   a    !  bxa
+ + + + ! + + + +
"
                        .parse::<Theorem>()
                        .unwrap();

        let deg180 = r"
+ + + ! + + +
      !  B
+ + + ! + + +
 3 a  !  3|a
+ + + ! +-+x+
 a    !  axb
+ + + ! + + +
"
                         .parse::<Theorem>()
                         .unwrap();

        let deg270 = r"
+ + + + ! + + + +
   a    !    axb
+ + + + ! + +-+x+
   3 a  !  B 3|a
+ + + + ! + + + +
"
                         .parse::<Theorem>()
                         .unwrap();

        let h_flip = r"
+ + + ! + + +
 a    !  axb
+ + + ! +-+x+
 3 a  !  3|a
+ + + ! + + +
      !  B
+ + + ! + + +
"
                         .parse::<Theorem>()
                         .unwrap();

        let v_flip = r"
+ + + ! + + +
      !    B
+ + + ! + + +
 a 3  !  a|3
+ + + ! +x+-+
   a  !  bxa
+ + + ! + + +
"
                         .parse::<Theorem>()
                         .unwrap();

        assert_eq!(deg0.clone(), deg0.clone().rotate(Rotation::UCW0));
        assert_eq!(deg90.clone(), deg0.clone().rotate(Rotation::UCW90));
        assert_eq!(deg180.clone(), deg0.clone().rotate(Rotation::UCW180));
        assert_eq!(deg270.clone(), deg0.clone().rotate(Rotation::UCW270));
        assert_eq!(h_flip.clone(), deg0.clone().rotate(Rotation::H_FLIP));
        assert_eq!(v_flip.clone(), deg0.clone().rotate(Rotation::V_FLIP));
        assert_eq!(v_flip.clone(), h_flip.clone().rotate(Rotation::UCW180));

        let mut rots = &mut [deg0.clone(),
                             deg90,
                             deg180,
                             deg270,
                             h_flip.clone(),
                             h_flip.clone().rotate(Rotation::UCW90),
                             h_flip.clone().rotate(Rotation::UCW180),
                             h_flip.clone().rotate(Rotation::UCW270)];
        rots.sort();
        assert_eq!(rots, &deg0.all_rotations()[..]);
    }

    #[test]
    fn all_rotations() {
        let theo = r"
+ + ! +x+
 0  ! x0x
+ + ! +x+
"
                       .parse::<Theorem>()
                       .unwrap();
        let rots = theo.clone().all_rotations();
        assert_eq!(&[theo], &rots[..]);
    }
}
