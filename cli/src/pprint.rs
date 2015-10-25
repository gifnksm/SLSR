use std::io;
use std::io::prelude::*;
use ansi_term::{Colour, Style};
use libc;
use slsr_core::puzzle::{Puzzle, Edge, Side};
use slsr_core::geom::{Geom, Point, Move};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Mode {
    Color,
    Ascii,
}

#[derive(Copy, Clone, Debug)]
pub struct Config {
    pub mode: Mode,
    pub cell_width: usize,
    pub cell_height: usize,
}

fn side_to_style(ty: Option<Side>) -> Style {
    match ty {
        Some(Side::In) => Colour::Black.on(Colour::Yellow),
        Some(Side::Out) => Colour::White.on(Colour::Black),
        None => Colour::Black.on(Colour::White),
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

struct Printer<T> {
    is_color: bool,
    out: T,
}

impl<T> Printer<T>
    where T: Write
{
    fn new(is_color: bool, out: T) -> Printer<T> {
        Printer {
            is_color: is_color,
            out: out,
        }
    }

    fn write_pretty(&mut self, side: Option<Side>, s: &str) -> io::Result<()> {
        if self.is_color {
            self.out.write_fmt(format_args!("{}", side_to_style(side).paint(s)))
        } else {
            self.out.write_all(s.as_bytes())
        }
    }

    fn write_plain(&mut self, s: &str) -> io::Result<()> {
        self.out.write_all(s.as_bytes())
    }
}

struct Table;
impl Table {
    fn pprint<T>(printer: &mut Printer<T>, conf: &Config, puzzle: &Puzzle) -> io::Result<()>
        where T: Write
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
    fn pprint<T>(printer: &mut Printer<T>, conf: &Config, puzzle: &Puzzle) -> io::Result<()>
        where T: Write
    {
        try!(printer.write_plain(&format!("{:1$}", "", conf.cell_width)));
        for x in 0..puzzle.column() {
            try!(printer.write_plain(&format!("{:1$}", "", 1)));
            try!(Label::pprint(printer, conf, x, true));
        }
        try!(printer.write_plain("\n"));
        Ok(())
    }
}

struct EdgeRow;
impl EdgeRow {
    fn pprint<T>(printer: &mut Printer<T>, conf: &Config, puzzle: &Puzzle, y: i32) -> io::Result<()>
        where T: Write
    {
        let col = puzzle.column();
        try!(printer.write_plain(&format!("{:1$}", "", conf.cell_width)));
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
    fn pprint<T>(printer: &mut Printer<T>, conf: &Config, puzzle: &Puzzle, y: i32) -> io::Result<()>
        where T: Write
    {
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
    fn pprint<T>(printer: &mut Printer<T>,
                 _conf: &Config,
                 puzzle: &Puzzle,
                 p: Point)
                 -> io::Result<()>
        where T: Write
    {
        let l = p + Move::LEFT;
        let u = p + Move::UP;
        let eh_p = puzzle.edge_h(p);
        let eh_l = puzzle.edge_h(l);
        let ev_p = puzzle.edge_v(p);
        let ev_u = puzzle.edge_v(u);

        let is_same_all = (eh_p == Some(Edge::Cross)) && (eh_l == Some(Edge::Cross)) &&
                          (ev_p == Some((Edge::Cross))) &&
                          (ev_u == Some((Edge::Cross)));

        let ty = if is_same_all {
            puzzle.side(p)
        } else {
            None
        };
        let is_h = eh_p == Some(Edge::Line) && eh_l == Some(Edge::Line);
        let is_v = ev_p == Some(Edge::Line) && ev_u == Some(Edge::Line);

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
    fn pprint<T>(printer: &mut Printer<T>,
                 conf: &Config,
                 puzzle: &Puzzle,
                 p: Point)
                 -> io::Result<()>
        where T: Write
    {
        let (s, ty) = match puzzle.edge_h(p) {
            Some(Edge::Cross) => (" ", puzzle.side(p)),
            Some(Edge::Line) => ("-", None),
            None => ("~", None),
        };
        for _ in 0..conf.cell_width {
            try!(printer.write_pretty(ty, s));
        }
        Ok(())
    }
}

struct Label;
impl Label {
    fn pprint<T>(printer: &mut Printer<T>, conf: &Config, n: i32, num_line: bool) -> io::Result<()>
        where T: Write
    {
        if num_line {
            let order = 10i32.pow(conf.cell_width as u32);
            try!(printer.write_plain(&format!("{:^1$}", n % order, conf.cell_width)));
        } else {
            try!(printer.write_plain(&format!("{:1$}", "", conf.cell_width)));
        }
        Ok(())
    }
}

struct EdgeV;
impl EdgeV {
    fn pprint<T>(printer: &mut Printer<T>,
                 _conf: &Config,
                 puzzle: &Puzzle,
                 p: Point)
                 -> io::Result<()>
        where T: Write
    {
        let (s, ty) = match puzzle.edge_v(p) {
            Some(Edge::Cross) => (" ", puzzle.side(p)),
            Some(Edge::Line) => ("|", None),
            None => ("?", None),
        };
        try!(printer.write_pretty(ty, s));
        Ok(())
    }
}

struct Cell;
impl Cell {
    fn pprint<T>(printer: &mut Printer<T>,
                 conf: &Config,
                 puzzle: &Puzzle,
                 p: Point,
                 num_line: bool)
                 -> io::Result<()>
        where T: Write
    {
        let ty = puzzle.side(p);
        match puzzle.hint(p) {
            Some(x) if num_line => {
                try!(printer.write_pretty(ty, &format!("{:^1$}", x, conf.cell_width)));
            }
            _ => {
                try!(printer.write_pretty(ty, &format!("{:^1$}", "", conf.cell_width)));
            }
        }
        Ok(())
    }
}

pub fn print(conf: &Config, puzzle: &Puzzle) -> io::Result<()> {
    let is_color = conf.mode == Mode::Color;
    Table::pprint(&mut Printer::new(is_color, io::stdout()), conf, puzzle)
}
