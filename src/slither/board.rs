use std::{fmt, iter};
use std::ops::{Index, IndexMut};
use std::str::FromStr;

use geom::{Geom, Point, Size, Matrix};

pub type Hint = Option<u8>;
#[derive(Copy, Clone, Show, Eq, PartialEq)]
pub enum Side { In, Out }
#[derive(Copy, Clone, Show, Eq, PartialEq)]
pub enum Edge { Line, Cross }

#[derive(Clone, Show, Eq, PartialEq)]
pub struct Board {
    size: Size,
    hint: Matrix<Hint>,
    side: Matrix<Option<Side>>,
    edge_v: Matrix<Option<Edge>>,
    edge_h: Matrix<Option<Edge>>
}

impl Board {
    pub fn new(size: Size) -> Board {
        assert!(size.0 > 0 && size.1 > 0);
        let hint = Matrix::new_empty(size, None, None);
        let side = Matrix::new_empty(size, Some(Side::Out), None);
        let edge_v = Matrix::new_empty(Size(size.0, size.1 + 1), Some(Edge::Cross), None);
        let edge_h = Matrix::new_empty(Size(size.0 + 1, size.1), Some(Edge::Cross), None);
        Board { size: size, hint: hint, side: side, edge_v: edge_v, edge_h: edge_h }
    }

    fn with_data(size: Size, hint: Vec<Hint>, side: Vec<Option<Side>>,
                 edge_v: Vec<Option<Edge>>, edge_h: Vec<Option<Edge>>) -> Board {
        assert!(size.0 > 0 && size.1 > 0);
        let hint = Matrix::new(size, None, hint);
        let side = Matrix::new(size, Some(Side::Out), side);
        let edge_v = Matrix::new(Size(size.0, size.1 + 1), Some(Edge::Cross), edge_v);
        let edge_h = Matrix::new(Size(size.0 + 1, size.1), Some(Edge::Cross), edge_h);
        Board { size: size, hint: hint, side: side, edge_v: edge_v, edge_h: edge_h }
    }

    pub fn hint(&self) -> &Matrix<Hint> { &self.hint }
    pub fn side(&self) -> &Matrix<Option<Side>> { &self.side }
    pub fn edge_h(&self) -> &Matrix<Option<Edge>> { &self.edge_h }
    pub fn edge_v(&self) -> &Matrix<Option<Edge>> { &self.edge_v }

    pub fn hint_mut(&mut self) -> &mut Matrix<Hint> { &mut self.hint }
    pub fn side_mut(&mut self) -> &mut Matrix<Option<Side>> { &mut self.side }
    pub fn edge_h_mut(&mut self) -> &mut Matrix<Option<Edge>> { &mut self.edge_h }
    pub fn edge_v_mut(&mut self) -> &mut Matrix<Option<Edge>> { &mut self.edge_v }
}

impl Geom for Board {
    fn size(&self) -> Size { self.size }
}

impl Index<Point> for Board {
    type Output = Hint;

    fn index(&self, p: &Point) -> &Hint {
        &self.hint[*p]
    }
}

impl IndexMut<Point> for Board {
    type Output = Hint;

    fn index_mut(&mut self, p: &Point) -> &mut Hint {
        &mut self.hint[*p]
    }
}

impl FromStr for Board {
    fn from_str(s: &str) -> Option<Board> {
        let mut mat = s.lines()
            .map(|l| l.trim_matches('\n'))
            .map(|l| l.chars().collect::<Vec<_>>())
            .skip_while(|l| l.is_empty())
            .collect::<Vec<_>>();

        while mat.last().map(|l| l.len()) == Some(0) {
            let _ = mat.pop();
        }

        if mat.len() == 0 { return None }

        fn parse_pat1(mat: Vec<Vec<char>>) -> Option<Board> {
            let rows = mat.iter()
                .enumerate()
                .filter(|&(_, cs)| cs[0] == '+')
                .map(|(i, _)| i)
                .collect::<Vec<_>>();
            let cols = mat[rows[0]].iter()
                .enumerate()
                .filter(|&(_, &c)| c == '+')
                .map(|(i, _)| i)
                .collect::<Vec<_>>();

            if rows.len() <= 1 { return None }
            if cols.len() <= 1 { return None }

            for &r in rows.iter() {
                if mat[r].iter().position(|&c| c == '+') != Some(cols[0]) {
                    return None
                }
                if mat[r].iter().rposition(|&c| c == '+') != Some(cols[cols.len() - 1]) {
                    return None
                }
                for &c in cols.iter() {
                    if mat[r].len() <= c {
                        return None
                    }
                    if mat[r][c] != '+' {
                        return None
                    }
                }
            }

            let mut edge_v = vec![];
            for (&rs, &re) in rows.iter().zip(rows[1 ..].iter()) {
                for &c in cols.iter() {
                    let s = mat[rs + 1 .. re]
                        .iter()
                        .map(|row| if c < row.len() { row[c] } else { ' ' })
                        .collect::<String>();
                    let edge = if s.len() == 0 {
                        None
                    } else if s.chars().all(|c| c == 'x') {
                        Some(Edge::Cross)
                    } else if s.chars().all(|c| c == '|') {
                        Some(Edge::Line)
                    } else {
                        None
                    };
                    edge_v.push(edge);
                }
            }

            let mut edge_h = vec!{};
            for &r in rows.iter() {
                for (&cs, &ce) in cols.iter().zip(cols[1 ..].iter()) {
                    let s = mat[r][cs + 1 .. ce]
                        .iter()
                        .cloned()
                        .collect::<String>();
                    let edge = if s.len() == 0 {
                        None
                    } else if s.chars().all(|c| c == 'x') {
                        Some(Edge::Cross)
                    } else if s.chars().all(|c| c == '-') {
                        Some(Edge::Line)
                    } else {
                        None
                    };
                    edge_h.push(edge);
                }
            }

            let mut hint = vec![];
            for (&rs, &re) in rows.iter().zip(rows[1 ..].iter()) {
                for (&cs, &ce) in cols.iter().zip(cols[1 ..].iter()) {
                    let s = mat[rs + 1 .. re]
                        .iter()
                        .flat_map(|row| row[cs + 1 .. ce].iter())
                        .cloned()
                        .collect::<String>();
                    let cell = match s.trim_matches(' ') {
                        "0" => Some(0),
                        "1" => Some(1),
                        "2" => Some(2),
                        "3" => Some(3),
                        _ => None
                    };
                    hint.push(cell);
                }
            }

            let size = Size((rows.len() - 1) as i32, (cols.len() - 1) as i32);
            let side = iter::repeat(None).take((rows.len() - 1) * (cols.len() - 1)).collect();
            Some(Board::with_data(size, hint, side, edge_v, edge_h))
        }

        fn parse_pat2(mat: Vec<Vec<char>>) -> Option<Board> {
            let row = mat.len();
            if row == 0 { return None }
            let col = mat[0].len();
            if col == 0 { return None }
            if mat[1 ..].iter().any(|r| r.len() != col) { return None }

            let hint = mat.iter().flat_map(|line| {
                line.iter().filter_map(|&c| {
                    match c {
                        '0' => Some(Some(0)),
                        '1' => Some(Some(1)),
                        '2' => Some(Some(2)),
                        '3' => Some(Some(3)),
                        '_' | '-' => Some(None),
                        _ => None
                    }
                })
            }).collect::<Vec<_>>();
            if hint.len() != row * col { return None }

            let size = Size(row as i32, col as i32);
            let side = iter::repeat(None).take(row * col).collect();
            let edge_v = iter::repeat(None).take(row * (col + 1)).collect();
            let edge_h = iter::repeat(None).take((row + 1) * col).collect();
            Some(Board::with_data(size, hint, side, edge_v, edge_h))
        }

        if mat[0].iter().any(|&c| c == '+') {
            parse_pat1(mat)
        } else {
            parse_pat2(mat)
        }
    }
}

struct Cross;
impl fmt::String for Cross {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "+")
    }
}

struct HEdge<'a>(&'a Board, Point);
impl<'a> fmt::String for HEdge<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let HEdge(board, p) = *self;
        match board.edge_h[p] {
            Some(Edge::Cross) => try!(write!(f, "x")),
            Some(Edge::Line) => try!(write!(f, "-")),
            None => try!(write!(f, " "))
        }
        Ok(())
    }
}

struct VEdge<'a>(&'a Board, Point);
impl<'a> fmt::String for VEdge<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let VEdge(board, p) = *self;
        match board.edge_v[p] {
            Some(Edge::Cross) => try!(write!(f, "x")),
            Some(Edge::Line) => try!(write!(f, "|")),
            None => try!(write!(f, " "))
        }
        Ok(())
    }
}

struct EdgeRow<'a>(&'a Board, i32);
impl<'a> fmt::String for EdgeRow<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let EdgeRow(board, r) = *self;
        for c in (0 .. board.column()) {
            let p = Point(r, c);
            try!(write!(f, "{}", Cross));
            try!(write!(f, "{}", HEdge(board, p)));
        }
        try!(write!(f, "{}", Cross));
        Ok(())
    }
}

struct CellRow<'a>(&'a Board, i32);
impl<'a> fmt::String for CellRow<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let CellRow(board, r) = *self;
        for c in (0 .. board.column()) {
            let p = Point(r, c);
            try!(write!(f, "{}", VEdge(board, p)));
            match board.hint[p] {
                Some(n) => try!(write!(f, "{}", n)),
                None => try!(write!(f, " "))
            }
        }
        try!(write!(f, "{}", VEdge(board, Point(r, board.column()))));
        Ok(())
    }
}

impl fmt::String for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for r in (0 .. self.row()) {
            try!(writeln!(f, "{}", EdgeRow(self, r)));
            try!(writeln!(f, "{}", CellRow(self, r)));
        }
        try!(writeln!(f, "{}", EdgeRow(self, self.row())));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Board;
    use geom::{Geom, Size, Point};

    #[test]
    fn parse() {
        let input = "123___
______
3_____
";
        let hint = input.parse::<Board>().unwrap();
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

        assert_eq!(Some(&hint), hint.to_string().parse::<Board>().as_ref());

        assert_eq!(None, "1243".parse::<Board>());

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

        let hint = input.parse::<Board>().unwrap();
        assert_eq!(Some(1), hint[Point(1, 1)]);
        assert_eq!(Some(2), hint[Point(1, 3)]);
        assert_eq!(output, hint.to_string());

        assert_eq!(None, "".parse::<Board>());

        let input = "
+ + + +
 1 2 3
+ + + +
";
        let hint = input.parse::<Board>().unwrap();
        assert_eq!(Some(1), hint[Point(0, 0)]);
        assert_eq!(Some(2), hint[Point(0, 1)]);
        assert_eq!(Some(3), hint[Point(0, 2)]);
    }
}
