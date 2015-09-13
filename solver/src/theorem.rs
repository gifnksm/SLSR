use std::str::FromStr;
use slsr_core::board::Edge;
use slsr_core::geom::{Point, Rotation, Move, Size};
use slsr_core::lattice_parser::LatticeParser;

use super::{State, SolverResult, LogicError};
use side_map::SideMap;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Pattern {
    Hint(u8, Point),
    Edge(Edge, Point, Point)
}

enum PatternMatch {
    Complete, Partial, Conflict
}

impl Pattern {
    fn hint(h: u8, p: Point) -> Pattern {
        Pattern::Hint(h, p).normalized()
    }
    fn cross(p0: Point, p1: Point) -> Pattern {
        Pattern::Edge(Edge::Cross, p0, p1).normalized()
    }
    fn line(p0: Point, p1: Point) -> Pattern {
        Pattern::Edge(Edge::Line, p0, p1).normalized()
    }

    fn normalized(self) -> Pattern {
        match self {
            Pattern::Edge(edge, p0, p1) if p1 < p0 => {
                Pattern::Edge(edge, p1, p0)
            }
            x => x
        }
    }

    fn rotate(self, rot: Rotation) -> Pattern {
        let o = Point(0, 0);
        match self {
            Pattern::Hint(h, p) => { Pattern::Hint(h, o + rot * (p - o)) }
            Pattern::Edge(e, p0, p1) => {
                Pattern::Edge(e, o + rot * (p0 - o), o + rot * (p1 - o))
            }
        }.normalized()
    }

    fn shift(self, d: Move) -> Pattern {
        match self {
            Pattern::Hint(h, p) => { Pattern::Hint(h, p + d) }
            Pattern::Edge(e, p0, p1) => {
                Pattern::Edge(e, p0 + d, p1 + d)
            }
        }.normalized()
    }

    fn matches(self, side_map: &mut SideMap) -> SolverResult<PatternMatch> {
        Ok(match self {
            Pattern::Hint(h, p) => {
                if side_map.hint()[p] == Some(h) {
                    PatternMatch::Complete
                } else {
                    PatternMatch::Conflict
                }
            }
            Pattern::Edge(e, p0, p1) => {
                match side_map.get_edge(p0, p1) {
                    State::Fixed(edg) => {
                        if e == edg {
                            PatternMatch::Complete
                        } else {
                            PatternMatch::Conflict
                        }
                    }
                    State::Unknown => PatternMatch::Partial,
                    State::Conflict => return Err(LogicError)
                }
            }
        })
    }

    pub fn apply(&self, side_map: &mut SideMap) {
        match self {
            &Pattern::Hint(..) => panic!(),
            &Pattern::Edge(e, p0, p1) => {
                let _ = side_map.set_edge(p0, p1, e);
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Theorem {
    size: Size,
    matcher: Vec<Pattern>,
    result: Vec<Pattern>
}

pub enum TheoremMatch {
    Complete(Vec<Pattern>),
    Partial(Theorem),
    Conflict
}

impl Theorem {
    fn normalized(mut self) -> Theorem {
        self.matcher.sort();
        self.matcher.dedup();
        self.result.sort();
        self.result.dedup();
        self
    }

    fn rotate(mut self, rot: Rotation) -> Theorem {
        let size = rot * Move(self.size.0, self.size.1);

        let mut d = Move(0, 0);
        if size.0 < 0 { d = d + Move(- size.0 - 1, 0); }
        if size.1 < 0 { d = d + Move(0, - size.1 - 1); }

        self.size = Size(size.0.abs(), size.1.abs());
        for x in self.matcher.iter_mut() {
            *x = x.rotate(rot).shift(d);
        }
        for x in self.result.iter_mut() {
            *x = x.rotate(rot).shift(d)
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

        self
    }

    pub fn all_rotations(self) -> Vec<Theorem> {
        let deg90  = self.clone().rotate(Rotation::UCW90);
        let deg180 = self.clone().rotate(Rotation::UCW180);
        let deg270 = self.clone().rotate(Rotation::UCW270);
        let h_deg0   = self.clone().rotate(Rotation::H_FLIP);
        let h_deg90  = h_deg0.clone().rotate(Rotation::UCW90);
        let h_deg180 = h_deg0.clone().rotate(Rotation::UCW180);
        let h_deg270 = h_deg0.clone().rotate(Rotation::UCW270);
        let mut rots = vec![self.clone(), deg90, deg180, deg270,
                            h_deg0, h_deg90, h_deg180, h_deg270];
        rots.sort();
        // FIXME: Should reduce the elements that has different result but size
        //        and matcher are same.
        rots.dedup();

        rots
    }

    pub fn size(&self) -> Size { self.size }
    pub fn head(&self) -> Pattern { self.matcher[0] }

    pub fn matches(mut self, side_map: &mut SideMap)
                   -> SolverResult<TheoremMatch>
    {
        let mut w = 0;
        for r in (0 .. self.matcher.len()) {
            let read = self.matcher[r];
            match try!(read.matches(side_map)) {
                PatternMatch::Complete => {},
                PatternMatch::Partial => {
                    self.matcher[w] = read;
                    w += 1;
                }
                PatternMatch::Conflict => {
                    return Ok(TheoremMatch::Conflict)
                }
            }
        }
        unsafe { self.matcher.set_len(w); }

        let m = if self.matcher.is_empty() {
            TheoremMatch::Complete(self.result)
        } else {
            TheoremMatch::Partial(self)
        };
        Ok(m)
    }
}

impl FromStr for Theorem {
    type Err = ();

    fn from_str(s: &str) -> Result<Theorem, ()> {
        let mut matcher_lines = vec![];
        let mut result_lines = vec![];

        let lines = s.lines()
            .map(|l| l.trim_matches('\n'))
            .filter(|s| !s.is_empty());

        for line in lines {
            let mut it = line.splitn(2, '!');
            if let Some(l) = it.next() {
                matcher_lines.push(l.chars().collect());
            } else {
                return Err(())
            }

            if let Some(l) = it.next() {
                result_lines.push(l.chars().collect());
            } else {
                return Err(())
            }
        }

        let (m_size, m_pat) = try!(parse_lines(&matcher_lines));
        let (r_size, mut r_pat) = try!(parse_lines(&result_lines));
        if m_size != r_size { return Err(()) }

        let mut idx = 0;
        for &p in &m_pat {
            match r_pat[idx ..].iter().position(|&x| x == p) {
                Some(i) => {
                    idx += i;
                    let _ = r_pat.remove(idx);
                }
                None => { return Err(()) }
            }
        }

        return Ok(Theorem { size: m_size, matcher: m_pat, result: r_pat });

        fn parse_lines(lines: &[Vec<char>]) -> Result<(Size, Vec<Pattern>), ()> {
            let parser = match LatticeParser::new(lines) {
                Some(x) => x, None => return Err(())
            };

            let rows = parser.num_rows();
            let cols = parser.num_cols();

            if rows <= 1 { return Err(()) }
            if cols <= 1 { return Err(()) }

            let size = Size((rows - 1) as i32, (cols - 1) as i32);

            let mut pat = vec![];

            for (p, s) in parser.v_edges() {
                if s.is_empty() {
                    continue
                }
                if s.chars().all(|c| c == 'x') {
                    pat.push(Pattern::cross(p + Move::LEFT, p));
                    continue
                }
                if s.chars().all(|c| c == '|') {
                    pat.push(Pattern::line(p + Move::LEFT, p));
                    continue
                }
            }

            for (p, s) in parser.h_edges() {
                if s.is_empty() {
                    continue
                }
                if s.chars().all(|c| c == 'x') {
                    pat.push(Pattern::cross(p + Move::UP, p));
                    continue
                }
                if s.chars().all(|c| c == '-') {
                    pat.push(Pattern::line(p + Move::UP, p));
                    continue
                }
            }

            let mut pairs: Vec<(char, Vec<Point>, Vec<Point>)> = vec![];

            for (p, s) in parser.cells() {
                for c in s.trim_matches(' ').chars() {
                    match c {
                        '0' => { pat.push(Pattern::hint(0, p)); }
                        '1' => { pat.push(Pattern::hint(1, p)); }
                        '2' => { pat.push(Pattern::hint(2, p)); }
                        '3' => { pat.push(Pattern::hint(3, p)); }
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
                    for &p in &ps0[1 ..] {
                        pat.push(Pattern::cross(ps0[0], p));
                    }
                }
                if ps1.len() > 0 {
                    for &p in &ps1[1 ..] {
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
    use super::{Pattern, Theorem};

    #[test]
    fn parse() {
        fn check(size: Size, mut matcher: Vec<Pattern>, mut result: Vec<Pattern>,
                 input: &str)
        {
            for p in matcher.iter_mut() { *p = p.normalized(); }
            for p in result.iter_mut() { *p = p.normalized(); }
            matcher.sort();
            matcher.dedup();
            result.sort();
            result.dedup();
            assert_eq!(Ok(Theorem { size: size, matcher: matcher, result: result }),
                       input.parse::<Theorem>())
        }

        check(Size(1, 1),
              vec![Pattern::hint(0, Point(0, 0))],
              vec![Pattern::cross(Point(0, 0), Point(0, -1)),
                   Pattern::cross(Point(0, 0), Point(0, 1)),
                   Pattern::cross(Point(0, 0), Point(-1, 0)),
                   Pattern::cross(Point(0, 0), Point(1, 0))],"
+ + ! +x+
 0  ! x0x
+ + ! +x+
");
        check(Size(3, 3),
              vec![Pattern::hint(0, Point(1, 0)),
                   Pattern::hint(3, Point(1, 1))],
              vec![Pattern::cross(Point(1, 0), Point(1, -1)),
                   Pattern::cross(Point(1, 0), Point(1, 1)),
                   Pattern::cross(Point(1, 0), Point(0, 0)),
                   Pattern::cross(Point(1, 0), Point(2, 0)),
                   Pattern::cross(Point(0, 1), Point(0, 2)),
                   Pattern::cross(Point(1, 2), Point(0, 2)),
                   Pattern::cross(Point(1, 2), Point(2, 2)),
                   Pattern::cross(Point(2, 1), Point(2, 2)),
                   Pattern::line(Point(0, 0), Point(0, 1)),
                   Pattern::line(Point(0, 1), Point(1, 1)),
                   Pattern::line(Point(1, 1), Point(1, 2)),
                   Pattern::line(Point(1, 1), Point(2, 1)),
                   Pattern::line(Point(2, 0), Point(2, 1))], "
+ + + + ! + + + +
        !   | x
+ + + + ! +x+-+x+
 0 3    ! x0x3|
+ + + + ! +x+-+x+
        !   | x
+ + + + ! + + + +
");
        check(Size(2, 2),
              vec![Pattern::hint(1, Point(1, 1)),
                   Pattern::line(Point(1, 0), Point(0, 1))],
              vec![Pattern::cross(Point(1, 1), Point(1, 2)),
                   Pattern::cross(Point(1, 1), Point(2, 1))], "
+ + + ! + + +
   a  !    a
+ + + ! + + +
 A 1  !  A 1x
+ + + ! + +x+
");
        check(Size(3, 3),
              vec![Pattern::hint(3, Point(1, 1)),
                   Pattern::cross(Point(1, 0), Point(0, 1))],
              vec![Pattern::cross(Point(0, 0), Point(0, 1)),
                   Pattern::cross(Point(0, 0), Point(1, 0)),
                   Pattern::line(Point(0, 1), Point(1, 1)),
                   Pattern::line(Point(1, 0), Point(1, 1)),
                   Pattern::line(Point(1, 2), Point(2, 1))], "
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
        let deg0 = "
+ + + ! + + +
   a  !  bxa
+ + + ! +x+-+
 a 3  !  a|3
+ + + ! + + +
      !    B
+ + + ! + + +
".parse::<Theorem>().unwrap();

        let deg90 = "
+ + + + ! + + + +
 a 3    !  a|3 B
+ + + + ! +x+-+ +
   a    !  bxa
+ + + + ! + + + +
".parse::<Theorem>().unwrap();

        let deg180 = "
+ + + ! + + +
      !  B
+ + + ! + + +
 3 a  !  3|a
+ + + ! +-+x+
 a    !  axb
+ + + ! + + +
".parse::<Theorem>().unwrap();

        let deg270 = "
+ + + + ! + + + +
   a    !    axb
+ + + + ! + +-+x+
   3 a  !  B 3|a
+ + + + ! + + + +
".parse::<Theorem>().unwrap();

        let h_flip = "
+ + + ! + + +
 a    !  axb
+ + + ! +-+x+
 3 a  !  3|a
+ + + ! + + +
      !  B
+ + + ! + + +
".parse::<Theorem>().unwrap();

        let v_flip = "
+ + + ! + + +
      !    B
+ + + ! + + +
 a 3  !  a|3
+ + + ! +x+-+
   a  !  bxa
+ + + ! + + +
".parse::<Theorem>().unwrap();

        assert_eq!(deg0.clone(), deg0.clone().rotate(Rotation::UCW0));
        assert_eq!(deg90.clone(), deg0.clone().rotate(Rotation::UCW90));
        assert_eq!(deg180.clone(), deg0.clone().rotate(Rotation::UCW180));
        assert_eq!(deg270.clone(), deg0.clone().rotate(Rotation::UCW270));
        assert_eq!(h_flip.clone(), deg0.clone().rotate(Rotation::H_FLIP));
        assert_eq!(v_flip.clone(), deg0.clone().rotate(Rotation::V_FLIP));
        assert_eq!(v_flip.clone(), h_flip.clone().rotate(Rotation::UCW180));

        let mut rots = &mut [deg0.clone(), deg90, deg180, deg270,
                             h_flip.clone(),
                             h_flip.clone().rotate(Rotation::UCW90),
                             h_flip.clone().rotate(Rotation::UCW180),
                             h_flip.clone().rotate(Rotation::UCW270)];
        rots.sort();
        assert_eq!(rots, &deg0.all_rotations()[..]);
    }

    #[test]
    fn all_rotations() {
        let theo = "
+ + ! +x+
 0  ! x0x
+ + ! +x+
".parse::<Theorem>().unwrap();
        let rots = theo.clone().all_rotations();
        assert_eq!(&[theo], &rots[..]);
    }
}
