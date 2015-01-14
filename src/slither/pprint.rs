use std::fmt;
use std::io::IoResult;
use std::io::stdio::{self, StdWriter};
use board::{Board, Edge, Side};
use geom::{Geom, Point, UP, LEFT};
use term::{self, Terminal, WriterWrapper};
use term::color::{self, Color};

enum Output<T> {
    Pretty(Box<Terminal<WriterWrapper> + Send>),
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
    output: Output<StdWriter>,
    board: &'a Board
}

impl<'a> Printer<'a> {
    fn new(board: &'a Board) -> Printer<'a> {
        let output = if stdio::stdout_raw().isatty() {
            match term::stdout() {
                Some(t) => Output::Pretty(t),
                None    => Output::Raw(stdio::stdout_raw())
            }
        } else {
            Output::Raw(stdio::stdout_raw())
        };
        Printer { output: output, board: board }
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

    fn print(&mut self) -> IoResult<()> {
        let row = self.board.row();
        try!(self.label_row());
        for y in (0 .. row) {
            try!(self.edge_row(y));
            try!(self.cell_row(y));
        }
        try!(self.edge_row(row));
        try!(self.label_row());
        Ok(())
    }

    fn label_row(&mut self) -> IoResult<()> {
        try!(self.write_plain("  "));
        for x in 0 .. self.board.column() {
            try!(self.write_plain_fmt(format_args!(" {:2}", x)));
        }
        try!(self.write_plain("\n"));
        Ok(())
    }

    fn edge_row(&mut self, y: i32) -> IoResult<()> {
        let col = self.board.column();
        try!(self.write_plain("  "));
        for x in (0 .. col) {
            try!(self.corner(Point(y, x)));
            try!(self.edge_h(Point(y, x)));
        }
        try!(self.corner(Point(y, col)));
        try!(self.write_plain("\n"));
        Ok(())
    }

    fn cell_row(&mut self, y: i32) -> IoResult<()> {
        let col = self.board.column();
        try!(self.write_plain_fmt(format_args!("{:2}", y)));
        for x in (0 .. col) {
            try!(self.edge_v(Point(y, x)));
            try!(self.cell(Point(y, x)));
        }
        try!(self.edge_v(Point(y, col)));
        try!(self.write_plain_fmt(format_args!("{:2}\n", y)));
        Ok(())
    }

    fn corner(&mut self, p: Point) -> IoResult<()> {
        let is_same_all =
            (self.board.edge_h()[p] == Some(Edge::Cross)) &&
            (self.board.edge_h()[p + LEFT] == Some(Edge::Cross)) &&
            (self.board.edge_v()[p] == Some((Edge::Cross))) &&
            (self.board.edge_v()[p + UP] == Some((Edge::Cross)));

        let ty = if is_same_all {
            self.board.side()[p]
        } else {
            None
        };
        try!(self.write_pretty(ty, "+"));
        Ok(())
    }

    fn edge_h(&mut self, p: Point) -> IoResult<()> {
        let (s, ty) = match self.board.edge_h()[p] {
            Some(Edge::Cross) => (" ", self.board.side()[p]),
            Some(Edge::Line)  => ("-", None),
            None => ("~", None)
        };
        try!(self.write_pretty_fmt(ty, format_args!("{}{}", s, s)));
        Ok(())
    }

    fn edge_v(&mut self, p: Point) -> IoResult<()> {
        let (s, ty) = match self.board.edge_v()[p] {
            Some(Edge::Cross) => (" ", self.board.side()[p]),
            Some(Edge::Line)  => ("|", None),
            None => ("?", None)
        };
        try!(self.write_pretty(ty, s));
        Ok(())
    }

    fn cell(&mut self, p: Point) -> IoResult<()> {
        let ty = self.board.side()[p];
        match self.board[p] {
            Some(x) => try!(self.write_pretty_fmt(ty, format_args!("{} ", x))),
            None    => try!(self.write_pretty(ty, "  "))
        }
        Ok(())
    }
}

pub fn print(board: &Board) -> IoResult<()> {
    Printer::new(board).print()
}

