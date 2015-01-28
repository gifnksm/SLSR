use std::str::FromStr;
use board::{Edge, Hint};
use geom::{Point, Size, LEFT, UP};
use util;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum Pattern {
    Hint(Hint, Point),
    Edge(Edge, Point, Point)
}

impl Pattern {
    fn normalized(self) -> Pattern {
        match self {
            Pattern::Edge(edge, p0, p1) if p1 < p0 => {
                Pattern::Edge(edge, p1, p0)
            }
            x => x
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Theorem {
    size: Size,
    matcher: Vec<Pattern>,
    result: Vec<Pattern>
}

impl FromStr for Theorem {
    fn from_str(s: &str) -> Option<Theorem> {
        let mut matcher_lines = vec![];
        let mut result_lines = vec![];

        let mut lines = s.lines()
            .map(|l| l.trim_matches('\n'))
            .filter(|s| !s.is_empty());

        for line in lines {
            let mut it = line.splitn(2, '!');
            if let Some(l) = it.next() {
                matcher_lines.push(l.chars().collect());
            } else {
                return None
            }

            if let Some(l) = it.next() {
                result_lines.push(l.chars().collect());
            } else {
                return None
            }
        }

        let (m_size, m_pat) = match parse_lines(&matcher_lines[]) {
            Some(x) => x, None => return None
        };
        let (r_size, mut r_pat) = match parse_lines(&result_lines[]) {
            Some(x) => x, None => return None
        };

        if m_size != r_size { return None }

        let mut idx = 0;
        for &p in m_pat.iter() {
            match r_pat[idx ..].iter().position(|&x| x == p) {
                Some(i) => {
                    idx += i;
                    let _ = r_pat.remove(idx);
                }
                None => { return None }
            }
        }

        return Some(Theorem { size: m_size, matcher: m_pat, result: r_pat });

        fn parse_lines(lines: &[Vec<char>]) -> Option<(Size, Vec<Pattern>)> {
            use util::{VEdges, HEdges, Cells};

            let (rows, cols) = match util::find_lattice(lines) {
                Some(x) => x, None => return None
            };

            if rows.len() <= 1 { return None }
            if cols.len() <= 1 { return None }

            let size = Size((rows.len() - 1) as i32, (cols.len() - 1) as i32);

            let mut pat = vec![];

            for (p, s) in VEdges::new(lines, &rows[], &cols[]) {
                if s.is_empty() {
                    continue
                }
                if s.chars().all(|c| c == 'x') {
                    pat.push(Pattern::Edge(Edge::Cross, p + LEFT, p));
                    continue
                }
                if s.chars().all(|c| c == '|') {
                    pat.push(Pattern::Edge(Edge::Line, p + LEFT, p));
                    continue
                }
            }

            for (p, s) in HEdges::new(lines, &rows[], &cols[]) {
                if s.is_empty() {
                    continue
                }
                if s.chars().all(|c| c == 'x') {
                    pat.push(Pattern::Edge(Edge::Cross, p + UP, p));
                    continue
                }
                if s.chars().all(|c| c == '-') {
                    pat.push(Pattern::Edge(Edge::Line, p + UP, p));
                    continue
                }
            }

            for (p, s) in Cells::new(lines, &rows[], &cols[]) {
                match s.trim_matches(' ') {
                    "" => {},
                    "0" => { pat.push(Pattern::Hint(Some(0), p)); }
                    "1" => { pat.push(Pattern::Hint(Some(1), p)); }
                    "2" => { pat.push(Pattern::Hint(Some(2), p)); }
                    "3" => { pat.push(Pattern::Hint(Some(3), p)); }
                    _ => {}
                }
            }

            let mut pat = pat.map_in_place(|p| p.normalized());
            pat.sort();
            Some((size, pat))
        }
    }
}

#[cfg(test)]
mod tests {
    use geom::{Point, Size};
    use board::Edge;
    use super::{Pattern, Theorem};

    #[test]
    fn parse() {
        fn check(size: Size, matcher: Vec<Pattern>, result: Vec<Pattern>,
                 input: &str)
        {
            let mut matcher = matcher.map_in_place(|p| p.normalized());
            let mut result = result.map_in_place(|p| p.normalized());
            matcher.sort();
            result.sort();
            assert_eq!(Some(Theorem { size: size, matcher: matcher, result: result }),
                       input.parse::<Theorem>())
        }

        check(Size(1, 1),
              vec![Pattern::Hint(Some(0), Point(0, 0))],
              vec![Pattern::Edge(Edge::Cross, Point(0, 0), Point(0, -1)),
                   Pattern::Edge(Edge::Cross, Point(0, 0), Point(-1, 0)),
                   Pattern::Edge(Edge::Cross, Point(0, 0), Point(0, 1)),
                   Pattern::Edge(Edge::Cross, Point(0, 0), Point(1, 0))],"
+ + ! +x+
 0  ! x0x
+ + ! +x+
");
    }
}
