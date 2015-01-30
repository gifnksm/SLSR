use std::fmt;
use std::default::Default;
use std::old_io::IoResult;
use std::old_io::stdio::{self, StdWriter};
use std::num::Int;
use board::{Board, Edge, Side};
use geom::{Geom, Point, UP, LEFT};
use term::{self, Terminal, StdTerminal};
use term::color::{self, Color};

#[derive(Copy, Debug, RustcDecodable)]
pub enum Mode {
    Ascii, Unicode
}
impl Default for Mode {
    fn default() -> Mode { Mode::Unicode }
}

pub struct Config {
    pub mode: Mode,
    pub is_cjk: bool,
    pub cell_width: usize,
    pub cell_height: usize
}

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
                try!(term.write_all(s.as_bytes()));
                try!(term.reset());
                Ok(())
            }
            Output::Raw(ref mut stdout) => stdout.write_all(s.as_bytes())
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
            Output::Pretty(ref mut term) => term.write_all(s.as_bytes()),
            Output::Raw(ref mut stdout) => stdout.write_all(s.as_bytes())
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
    fn pprint(printer: &mut Printer, conf: &Config, board: &Board)
              -> IoResult<()>
    {
        let row = board.row();
        try!(LabelRow::pprint(printer, conf, board));
        for y in (0 .. row) {
            try!(EdgeRow::pprint(printer, conf, board, y));
            try!(CellRow::pprint(printer, conf, board, y));
        }
        try!(EdgeRow::pprint(printer, conf, board, row));
        try!(LabelRow::pprint(printer, conf, board));
        Ok(())
    }
}

struct LabelRow;
impl LabelRow {
    fn pprint(printer: &mut Printer, conf: &Config, board: &Board)
              -> IoResult<()>
    {
        try!(printer.write_plain_fmt(
            format_args!("{:1$}", "", conf.cell_width)));
        for x in 0 .. board.column() {
            try!(printer.write_plain_fmt(
                format_args!("{:1$}", "", "─".width(conf.is_cjk))));
            try!(Label::pprint(printer, conf, x, true));
        }
        try!(printer.write_plain("\n"));
        Ok(())
   }
}

struct EdgeRow;
impl EdgeRow {
    fn pprint(printer: &mut Printer, conf: &Config, board: &Board, y: i32)
              -> IoResult<()>
    {
        let col = board.column();
        try!(printer.write_plain_fmt(
            format_args!("{:1$}", "", conf.cell_width)));
        for x in (0 .. col) {
            try!(Corner::pprint(printer, conf, board, Point(y, x)));
            try!(EdgeH::pprint(printer, conf, board, Point(y, x)));
        }
        try!(Corner::pprint(printer, conf, board, Point(y, col)));
        try!(printer.write_plain("\n"));
        Ok(())
    }
}

struct CellRow;
impl CellRow {
    fn pprint(printer: &mut Printer, conf: &Config, board: &Board, y: i32) -> IoResult<()> {
        let col = board.column();
        for i in (0 .. conf.cell_height) {
            let num_line = (conf.cell_height - 1) / 2 == i;
            try!(Label::pprint(printer, conf, y, num_line));
            for x in (0 .. col) {
                try!(EdgeV::pprint(printer, conf, board, Point(y, x)));
                try!(Cell::pprint(printer, conf, board, Point(y, x), num_line));
            }
            try!(EdgeV::pprint(printer, conf, board, Point(y, col)));
            try!(Label::pprint(printer, conf, y, num_line));
            try!(printer.write_plain("\n"));
        }
        Ok(())
    }
}

struct Corner;
impl Corner {
    fn pprint(printer: &mut Printer, conf: &Config, board: &Board, p: Point)
              -> IoResult<()>
    {
        let l = p + LEFT;
        let u = p + UP;
        let eh_p = board.edge_h()[p];
        let eh_l = board.edge_h()[l];
        let ev_p = board.edge_v()[p];
        let ev_u = board.edge_v()[u];

        let is_same_all =
            (eh_p == Some(Edge::Cross)) &&
            (eh_l == Some(Edge::Cross)) &&
            (ev_p == Some((Edge::Cross))) &&
            (ev_u == Some((Edge::Cross)));

        let ty = if is_same_all {
            board.side()[p]
        } else {
            None
        };
        let is_h = eh_p == Some(Edge::Line) &&
            eh_l == Some(Edge::Line);
        let is_v = ev_p == Some(Edge::Line) &&
            ev_u == Some(Edge::Line);

        match conf.mode {
            Mode::Ascii => {
                if is_same_all {
                    try!(printer.write_pretty(ty, "."));
                } else if is_h {
                    try!(printer.write_pretty(ty, "-"));
                } else if is_v {
                    try!(printer.write_pretty(ty, "|"));
                } else {
                    try!(printer.write_pretty(ty, "+"));
                }
            }
            Mode::Unicode => {
                let width = '┼'.width(conf.is_cjk).unwrap();
                if is_same_all {
                    for _ in (0 .. width) {
                        try!(printer.write_pretty(ty, " "));
                    }
                } else if is_h {
                    try!(printer.write_pretty(ty, "─"));
                } else if is_v {
                    try!(printer.write_pretty(ty, "│"));
                } else if eh_p == Some(Edge::Line) && ev_p == Some(Edge::Line) {
                    try!(printer.write_pretty(ty, "┌"));
                } else if eh_l == Some(Edge::Line) && ev_p == Some(Edge::Line) {
                    try!(printer.write_pretty(ty, "┐"));
                } else if eh_l == Some(Edge::Line) && ev_u == Some(Edge::Line) {
                    try!(printer.write_pretty(ty, "┘"));
                } else if eh_p == Some(Edge::Line) && ev_u == Some(Edge::Line) {
                    try!(printer.write_pretty(ty, "└"));
                } else {
                    try!(printer.write_pretty(ty, "┼"));
                }
            }
        }
        Ok(())
    }
}

struct EdgeH;
impl EdgeH {
    fn pprint(printer: &mut Printer, conf: &Config, board: &Board, p: Point)
              -> IoResult<()>
    {
        match conf.mode {
            Mode::Ascii => {
                let (s, ty) = match board.edge_h()[p] {
                    Some(Edge::Cross) => (" ", board.side()[p]),
                    Some(Edge::Line)  => ("-", None),
                    None => ("~", None)
                };
                for _ in (0 .. conf.cell_width) {
                    try!(printer.write_pretty(ty, s));
                }
            }
            Mode::Unicode => {
                let (s, ty) = match board.edge_h()[p] {
                    Some(Edge::Cross) => (" ", board.side()[p]),
                    Some(Edge::Line)  => ("─", None),
                    None => ("~", None)
                };
                let w = s.width(conf.is_cjk);
                for _ in (0 .. (conf.cell_width + w - 1) / w) {
                    try!(printer.write_pretty(ty, s));
                }
            }
        }
        Ok(())
    }
}

struct Label;
impl Label {
    fn pprint(printer: &mut Printer, conf: &Config, n: i32, num_line: bool)
              -> IoResult<()>
    {
        if num_line {
            let order = 10.pow(conf.cell_width);
            try!(printer.write_plain_fmt(
                format_args!("{:^1$}", n % order, conf.cell_width)));
        } else {
            try!(printer.write_plain_fmt(
                format_args!("{:1$}", "", conf.cell_width)));
        }
        Ok(())
    }
}

struct EdgeV;
impl EdgeV {
    fn pprint(printer: &mut Printer, conf: &Config, board: &Board, p: Point)
              -> IoResult<()>
    {
        match conf.mode {
            Mode::Ascii => {
                let (s, ty) = match board.edge_v()[p] {
                    Some(Edge::Cross) => (" ", board.side()[p]),
                    Some(Edge::Line)  => ("|", None),
                    None => ("?", None)
                };
                try!(printer.write_pretty(ty, s));
            }
            Mode::Unicode => {
                let width = "│".width(conf.is_cjk);
                let (s, ty) = match board.edge_v()[p] {
                    Some(Edge::Cross) => (" ", board.side()[p]),
                    Some(Edge::Line)  => ("│", None),
                    None => ("/", None)
                };
                for _ in (0 .. width / s.width(conf.is_cjk)) {
                    try!(printer.write_pretty(ty, s));
                }
            }
        }
        Ok(())
    }
}

struct Cell;
impl Cell {
    fn pprint(printer: &mut Printer, conf: &Config, board: &Board, p: Point,
              num_line: bool)
              -> IoResult<()>
    {
        let ty = board.side()[p];
        match board[p] {
            Some(x) if num_line => {
                try!(printer.write_pretty_fmt(
                    ty, format_args!("{:^1$}", x, conf.cell_width)));
            },
            _ => {
                try!(printer.write_pretty_fmt(
                    ty, format_args!("{:^1$}", "", conf.cell_width)));
            }
        };
        Ok(())
    }
}

pub fn print(conf: &Config, board: &Board) -> IoResult<()> {
    Table::pprint(&mut Printer::new(), conf, board)
}
