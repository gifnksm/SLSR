use std::fmt;
use std::io::{self, Stdout};
use std::io::prelude::*;
use libc;
use slsr_core::puzzle::{Puzzle, Edge, Side};
use slsr_core::geom::{Geom, Point, Move};
use term::{self, Terminal};
use term::color::{self, Color};

#[derive(Copy, Clone, Debug)]
pub enum Mode {
    Color, Ascii
}

#[derive(Copy, Clone, Debug)]
pub struct Config {
    pub mode: Mode,
    pub cell_width: usize,
    pub cell_height: usize
}

enum Output<T> {
    Pretty(Box<Terminal<Output=T> + Send>),
    Raw(T)
}

fn side_to_color(ty: Option<Side>) -> (Color, Color) {
    match ty {
        Some(Side::In)  => (color::BRIGHT_YELLOW, color::BLACK),
        Some(Side::Out) => (color::BLACK,         color::BRIGHT_WHITE),
        None            => (color::BRIGHT_WHITE,  color::BLACK),
    }
}

#[cfg(unix)]
fn isatty(fd: libc::c_int) -> bool {
    unsafe { libc::isatty(fd) != 0 }
}
#[cfg(windows)]
fn isatty(fd: libc::c_int) -> bool {
    extern crate kernel32;
    extern crate winapi;
    unsafe {
        let handle = kernel32::GetStdHandle(if fd == libc::STDOUT_FILENO {
            winapi::winbase::STD_OUTPUT_HANDLE
        } else {
            winapi::winbase::STD_ERROR_HANDLE
        });
        let mut out = 0;
        kernel32::GetConsoleMode(handle, &mut out) != 0
    }
}

pub fn is_pprintable() -> bool {
    isatty(libc::STDOUT_FILENO)
}

struct Printer {
    output: Output<Stdout>
}

impl Printer {
    fn new(conf: &Config) -> Printer {
        let output = match conf.mode {
            Mode::Color => {
                match term::stdout() {
                    Some(t) => Output::Pretty(t),
                    None    => Output::Raw(io::stdout())
                }
            }
            Mode::Ascii => Output::Raw(io::stdout())
        };
        Printer { output: output }
    }

    fn write_pretty(&mut self, ty: Option<Side>, s: &str) -> io::Result<()> {
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
    fn write_pretty_fmt(&mut self, ty: Option<Side>, fmt: fmt::Arguments) -> io::Result<()> {
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
    fn write_plain(&mut self, s: &str) -> io::Result<()> {
        match self.output {
            Output::Pretty(ref mut term) => term.write_all(s.as_bytes()),
            Output::Raw(ref mut stdout) => stdout.write_all(s.as_bytes())
        }
    }

    fn write_plain_fmt(&mut self, fmt: fmt::Arguments) -> io::Result<()> {
        match self.output {
            Output::Pretty(ref mut term) => term.write_fmt(fmt),
            Output::Raw(ref mut stdout) => stdout.write_fmt(fmt)
        }
    }
}

struct Table;
impl Table {
    fn pprint(printer: &mut Printer, conf: &Config, puzzle: &Puzzle)
              -> io::Result<()>
    {
        let row = puzzle.row();
        try!(LabelRow::pprint(printer, conf, puzzle));
        for y in 0..row {
            try!(EdgeRow::pprint(printer, conf, puzzle, y));
            try!(CellRow::pprint(printer, conf, puzzle, y));
        }
        try!(EdgeRow::pprint(printer, conf, puzzle, row));
        try!(LabelRow::pprint(printer, conf, puzzle));
        Ok(())
    }
}

struct LabelRow;
impl LabelRow {
    fn pprint(printer: &mut Printer, conf: &Config, puzzle: &Puzzle)
              -> io::Result<()>
    {
        try!(printer.write_plain_fmt(
            format_args!("{:1$}", "", conf.cell_width)));
        for x in 0..puzzle.column() {
            try!(printer.write_plain_fmt(
                format_args!("{:1$}", "", 1)));
            try!(Label::pprint(printer, conf, x, true));
        }
        try!(printer.write_plain("\n"));
        Ok(())
   }
}

struct EdgeRow;
impl EdgeRow {
    fn pprint(printer: &mut Printer, conf: &Config, puzzle: &Puzzle, y: i32)
              -> io::Result<()>
    {
        let col = puzzle.column();
        try!(printer.write_plain_fmt(
            format_args!("{:1$}", "", conf.cell_width)));
        for x in 0..col {
            try!(Corner::pprint(printer, conf, puzzle, Point(y, x)));
            try!(EdgeH::pprint(printer, conf, puzzle, Point(y, x)));
        }
        try!(Corner::pprint(printer, conf, puzzle, Point(y, col)));
        try!(printer.write_plain("\n"));
        Ok(())
    }
}

struct CellRow;
impl CellRow {
    fn pprint(printer: &mut Printer, conf: &Config, puzzle: &Puzzle, y: i32) -> io::Result<()> {
        let col = puzzle.column();
        for i in 0..conf.cell_height {
            let num_line = (conf.cell_height - 1) / 2 == i;
            try!(Label::pprint(printer, conf, y, num_line));
            for x in 0..col {
                try!(EdgeV::pprint(printer, conf, puzzle, Point(y, x)));
                try!(Cell::pprint(printer, conf, puzzle, Point(y, x), num_line));
            }
            try!(EdgeV::pprint(printer, conf, puzzle, Point(y, col)));
            try!(Label::pprint(printer, conf, y, num_line));
            try!(printer.write_plain("\n"));
        }
        Ok(())
    }
}

struct Corner;
impl Corner {
    fn pprint(printer: &mut Printer, _conf: &Config, puzzle: &Puzzle, p: Point)
              -> io::Result<()>
    {
        let l = p + Move::LEFT;
        let u = p + Move::UP;
        let eh_p = puzzle.edge_h(p);
        let eh_l = puzzle.edge_h(l);
        let ev_p = puzzle.edge_v(p);
        let ev_u = puzzle.edge_v(u);

        let is_same_all =
            (eh_p == Some(Edge::Cross)) &&
            (eh_l == Some(Edge::Cross)) &&
            (ev_p == Some((Edge::Cross))) &&
            (ev_u == Some((Edge::Cross)));

        let ty = if is_same_all {
            puzzle.side(p)
        } else {
            None
        };
        let is_h = eh_p == Some(Edge::Line) &&
            eh_l == Some(Edge::Line);
        let is_v = ev_p == Some(Edge::Line) &&
            ev_u == Some(Edge::Line);

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
    fn pprint(printer: &mut Printer, conf: &Config, puzzle: &Puzzle, p: Point)
              -> io::Result<()>
    {
        let (s, ty) = match puzzle.edge_h(p) {
            Some(Edge::Cross) => (" ", puzzle.side(p)),
            Some(Edge::Line)  => ("-", None),
            None => ("~", None)
        };
        for _ in 0..conf.cell_width {
            try!(printer.write_pretty(ty, s));
        }
        Ok(())
    }
}

struct Label;
impl Label {
    fn pprint(printer: &mut Printer, conf: &Config, n: i32, num_line: bool)
              -> io::Result<()>
    {
        if num_line {
            let order = 10i32.pow(conf.cell_width as u32);
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
    fn pprint(printer: &mut Printer, _conf: &Config, puzzle: &Puzzle, p: Point)
              -> io::Result<()>
    {
        let (s, ty) = match puzzle.edge_v(p) {
            Some(Edge::Cross) => (" ", puzzle.side(p)),
            Some(Edge::Line)  => ("|", None),
            None => ("?", None)
        };
        try!(printer.write_pretty(ty, s));
        Ok(())
    }
}

struct Cell;
impl Cell {
    fn pprint(printer: &mut Printer, conf: &Config, puzzle: &Puzzle, p: Point,
              num_line: bool)
              -> io::Result<()>
    {
        let ty = puzzle.side(p);
        match puzzle.hint(p) {
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

pub fn print(conf: &Config, puzzle: &Puzzle) -> io::Result<()> {
    Table::pprint(&mut Printer::new(conf), conf, puzzle)
}
