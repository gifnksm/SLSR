use std::fmt;
use std::io::IoResult;
use std::io::stdio::{self, StdWriter};
use board::{Board, Edge, Side};
use geom::{Geom, Point, UP, LEFT};
use term::{self, Terminal, StdTerminal};
use term::color::{self, Color};

enum Output<T> {
    Pretty(Box<StdTerminal>),
    Raw(T)
}

fn side_to_color(ty: Option<Side>) -> (Color, Color) {
    match ty {
        Some(Side::In)  => (color::BRIGHT_YELLOW, color::BLACK),
        Some(Side::Out) => (color::BLACK,         color::BRIGHT_WHITE),
        None            => (color::BRIGHT_WHITE,  color::BLACK),
    }
}

struct Printer<'a> {
    output: Output<StdWriter>
}

impl<'a> Printer<'a> {
    fn new() -> Printer<'a> {
        let output = if stdio::stdout_raw().isatty() {
            match term::stdout() {
                Some(t) => Output::Pretty(t),
                None    => Output::Raw(stdio::stdout_raw())
            }
        } else {
            Output::Raw(stdio::stdout_raw())
        };
        Printer { output: output }
    }

    fn write_pretty(&mut self, ty: Option<Side>, s: &str) -> IoResult<()> {
        match self.output {
            Output::Pretty(ref mut term) => {
                let (bg, fg) = side_to_color(ty);
                try!(term.fg(fg));
                try!(term.bg(bg));
                try!(term.write(s.as_bytes()));
                try!(term.reset());
                Ok(())
            }
            Output::Raw(ref mut stdout) => stdout.write(s.as_bytes())
        }
    }
    fn write_pretty_fmt(&mut self, ty: Option<Side>, fmt: fmt::Arguments) -> IoResult<()> {
        match self.output {
            Output::Pretty(ref mut term) => {
                let (bg, fg) = side_to_color(ty);
                try!(term.fg(fg));
                try!(term.bg(bg));
                try!(term.write_fmt(fmt));
                try!(term.reset());
                Ok(())
            }
            Output::Raw(ref mut stdout) => stdout.write_fmt(fmt)
        }
    }
    fn write_plain(&mut self, s: &str) -> IoResult<()> {
        match self.output {
            Output::Pretty(ref mut term) => term.write(s.as_bytes()),
            Output::Raw(ref mut stdout) => stdout.write(s.as_bytes())
        }
    }

    fn write_plain_fmt(&mut self, fmt: fmt::Arguments) -> IoResult<()> {
        match self.output {
            Output::Pretty(ref mut term) => term.write_fmt(fmt),
            Output::Raw(ref mut stdout) => stdout.write_fmt(fmt)
        }
    }
}

struct Table;
impl Table {
    fn pprint(printer: &mut Printer, board: &Board) -> IoResult<()> {
        let row = board.row();
        try!(LabelRow::pprint(printer, board));
        for y in (0 .. row) {
            try!(EdgeRow::pprint(printer, board, y));
            try!(CellRow::pprint(printer, board, y));
        }
        try!(EdgeRow::pprint(printer, board, row));
        try!(LabelRow::pprint(printer, board));
        Ok(())
    }
}

struct LabelRow;
impl LabelRow {
    fn pprint(printer: &mut Printer, board: &Board) -> IoResult<()> {
        try!(printer.write_plain("  "));
        for x in 0 .. board.column() {
            try!(printer.write_plain_fmt(format_args!(" {:2}", x)));
        }
        try!(printer.write_plain("\n"));
        Ok(())
   }
}

struct EdgeRow;
impl EdgeRow {
    fn pprint(printer: &mut Printer, board: &Board, y: i32) -> IoResult<()> {
        let col = board.column();
        try!(printer.write_plain("  "));
        for x in (0 .. col) {
            try!(Corner::pprint(printer, board, Point(y, x)));
            try!(EdgeH::pprint(printer, board, Point(y, x)));
        }
        try!(Corner::pprint(printer, board, Point(y, col)));
        try!(printer.write_plain("\n"));
        Ok(())
    }
}

struct CellRow;
impl CellRow {
    fn pprint(printer: &mut Printer, board: &Board, y: i32) -> IoResult<()> {
        let col = board.column();
        try!(printer.write_plain_fmt(format_args!("{:2}", y)));
        for x in (0 .. col) {
            try!(EdgeV::pprint(printer, board, Point(y, x)));
            try!(Cell::pprint(printer, board, Point(y, x)));
        }
        try!(EdgeV::pprint(printer, board, Point(y, col)));
        try!(printer.write_plain_fmt(format_args!("{:2}\n", y)));
        Ok(())
    }
}

struct Corner;
impl Corner {
    fn pprint(printer: &mut Printer, board: &Board, p: Point) -> IoResult<()> {
        let l = p + LEFT;
        let u = p + UP;
        let is_same_all =
            (board.edge_h()[p] == Some(Edge::Cross)) &&
            (board.edge_h()[l] == Some(Edge::Cross)) &&
            (board.edge_v()[p] == Some((Edge::Cross))) &&
            (board.edge_v()[u] == Some((Edge::Cross)));

        let ty = if is_same_all {
            board.side()[p]
        } else {
            None
        };
        let is_h = board.edge_h()[p] == Some(Edge::Line) &&
            board.edge_h()[l] == Some(Edge::Line);
        let is_v = board.edge_v()[p] == Some(Edge::Line) &&
            board.edge_v()[u] == Some(Edge::Line);

        if is_same_all {
            try!(printer.write_pretty(ty, "."));
        } else if is_h {
            try!(printer.write_pretty(ty, "-"));
        } else if is_v {
            try!(printer.write_pretty(ty, "|"));
        } else {
            try!(printer.write_pretty(ty, "+"));
        }
        Ok(())
    }
}

struct EdgeH;
impl EdgeH {
    fn pprint(printer: &mut Printer, board: &Board, p: Point) -> IoResult<()> {
        let (s, ty) = match board.edge_h()[p] {
            Some(Edge::Cross) => (" ", board.side()[p]),
            Some(Edge::Line)  => ("-", None),
            None => ("~", None)
        };
        try!(printer.write_pretty_fmt(ty, format_args!("{}{}", s, s)));
        Ok(())
    }
}

struct EdgeV;
impl EdgeV {
    fn pprint(printer: &mut Printer, board: &Board, p: Point) -> IoResult<()> {
        let (s, ty) = match board.edge_v()[p] {
            Some(Edge::Cross) => (" ", board.side()[p]),
            Some(Edge::Line)  => ("|", None),
            None => ("?", None)
        };
        try!(printer.write_pretty(ty, s));
        Ok(())
    }
}

struct Cell;
impl Cell {
    fn pprint(printer: &mut Printer, board: &Board, p: Point) -> IoResult<()> {
        let ty = board.side()[p];
        match board[p] {
            Some(x) => try!(printer.write_pretty_fmt(ty, format_args!("{} ", x))),
            None    => try!(printer.write_pretty(ty, "  "))
        }
        Ok(())
    }
}

pub fn print(board: &Board) -> IoResult<()> {
    Table::pprint(&mut Printer::new(), board)
}

