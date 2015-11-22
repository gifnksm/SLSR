use std::{io, iter};
use std::io::Stdout;
use std::io::prelude::*;
use term::{self, color, Terminal, StdoutTerminal};
use term::color::Color;
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

struct Style {
    fg: Color,
    bg: Color,
}

fn side_to_style(ty: Option<Side>) -> Style {
    match ty {
        Some(Side::In) => {
            Style {
                fg: color::BLACK,
                bg: color::YELLOW,
            }
        }
        Some(Side::Out) => {
            Style {
                fg: color::WHITE,
                bg: color::BLACK,
            }
        }
        None => {
            Style {
                fg: color::BLACK,
                bg: color::WHITE,
            }
        }
    }
}

#[cfg(unix)]
fn isatty_stdout() -> bool {
    extern crate libc;
    unsafe { libc::isatty(libc::STDOUT_FILENO) != 0 }
}
#[cfg(windows)]
fn isatty_stdout() -> bool {
    extern crate kernel32;
    extern crate winapi;
    unsafe {
        let handle = winapi::winbase::STD_OUTPUT_HANDLE as *mut winapi::c_void;
        let mut out = 0;
        kernel32::GetConsoleMode(handle, &mut out) != 0
    }
}

pub fn is_pprintable() -> bool {
    isatty_stdout()
}

trait Printer {
    fn write_pretty(&mut self, side: Option<Side>, s: &str) -> io::Result<()>;
    fn write_plain(&mut self, s: &str) -> io::Result<()>;
}

impl Printer for Stdout {
    fn write_pretty(&mut self, _side: Option<Side>, s: &str) -> io::Result<()> {
        self.write_all(s.as_bytes())
    }
    fn write_plain(&mut self, s: &str) -> io::Result<()> {
        self.write_all(s.as_bytes())
    }
}

impl Printer for Box<StdoutTerminal> {
    fn write_pretty(&mut self, side: Option<Side>, s: &str) -> io::Result<()> {
        let style = side_to_style(side);
        try!(self.reset());
        try!(self.fg(style.fg));
        try!(self.bg(style.bg));
        self.write_all(s.as_bytes())
    }
    fn write_plain(&mut self, s: &str) -> io::Result<()> {
        try!(self.reset());
        self.write_all(s.as_bytes())
    }
}

enum StdoutPrinter {
    Pretty(Box<StdoutTerminal>),
    Raw(Stdout),
}

impl StdoutPrinter {
    fn new(is_color: bool) -> StdoutPrinter {
        if !is_color {
            return StdoutPrinter::Raw(io::stdout());
        }

        match term::stdout() {
            Some(t) => StdoutPrinter::Pretty(t),
            None => StdoutPrinter::Raw(io::stdout()),
        }
    }
}

struct Table {
    label_row: LabelRow,
    edge_row: EdgeRow,
    cell_row: CellRow,
}

impl Table {
    fn new(conf: &Config) -> Table {
        Table {
            label_row: LabelRow::new(conf),
            edge_row: EdgeRow::new(conf),
            cell_row: CellRow::new(conf),
        }
    }

    fn pprint<P>(&self, printer: &mut P, puzzle: &Puzzle) -> io::Result<()>
        where P: Printer
    {
        let row = puzzle.row();
        try!(self.label_row.pprint(printer, puzzle));
        for y in 0..row {
            try!(self.edge_row.pprint(printer, puzzle, y));
            try!(self.cell_row.pprint(printer, puzzle, y));
        }
        try!(self.edge_row.pprint(printer, puzzle, row));
        try!(self.label_row.pprint(printer, puzzle));
        Ok(())
    }
}

struct LabelRow {
    space_left: String,
    space_cross: String,
    label: Label,
}

impl LabelRow {
    fn new(conf: &Config) -> LabelRow {
        LabelRow {
            space_left: format!("{:1$}", "", conf.cell_width),
            space_cross: format!("{:1$}", "", 1),
            label: Label::new(conf),
        }
    }

    fn pprint<P>(&self, printer: &mut P, puzzle: &Puzzle) -> io::Result<()>
        where P: Printer
    {
        try!(printer.write_plain(&self.space_left));
        for x in 0..puzzle.column() {
            try!(printer.write_plain(&self.space_cross));
            try!(self.label.pprint(printer, x, true));
        }
        try!(printer.write_plain("\n"));
        Ok(())
    }
}

struct EdgeRow {
    space_left: String,
    corner: Corner,
    edge_h: EdgeH,
}

impl EdgeRow {
    fn new(conf: &Config) -> EdgeRow {
        EdgeRow {
            space_left: format!("{:1$}", "", conf.cell_width),
            corner: Corner::new(conf),
            edge_h: EdgeH::new(conf),
        }
    }

    fn pprint<P>(&self, printer: &mut P, puzzle: &Puzzle, y: i32) -> io::Result<()>
        where P: Printer
    {
        let col = puzzle.column();
        try!(printer.write_plain(&self.space_left));
        for x in 0..col {
            try!(self.corner.pprint(printer, puzzle, Point(y, x)));
            try!(self.edge_h.pprint(printer, puzzle, Point(y, x)));
        }
        try!(self.corner.pprint(printer, puzzle, Point(y, col)));
        try!(printer.write_plain("\n"));
        Ok(())
    }
}

struct CellRow {
    cell_height: usize,
    edge_v: EdgeV,
    label: Label,
    cell: Cell,
}

impl CellRow {
    fn new(conf: &Config) -> CellRow {
        CellRow {
            cell_height: conf.cell_height,
            edge_v: EdgeV::new(conf),
            label: Label::new(conf),
            cell: Cell::new(conf),
        }
    }

    fn pprint<P>(&self, printer: &mut P, puzzle: &Puzzle, y: i32) -> io::Result<()>
        where P: Printer
    {
        let col = puzzle.column();
        for i in 0..self.cell_height {
            let num_line = (self.cell_height - 1) / 2 == i;
            try!(self.label.pprint(printer, y, num_line));
            for x in 0..col {
                try!(self.edge_v.pprint(printer, puzzle, Point(y, x)));
                try!(self.cell.pprint(printer, puzzle, Point(y, x), num_line));
            }
            try!(self.edge_v.pprint(printer, puzzle, Point(y, col)));
            try!(self.label.pprint(printer, y, num_line));
            try!(printer.write_plain("\n"));
        }
        Ok(())
    }
}

struct Corner;
impl Corner {
    fn new(_conf: &Config) -> Corner {
        Corner
    }

    fn pprint<P>(&self, printer: &mut P, puzzle: &Puzzle, p: Point) -> io::Result<()>
        where P: Printer
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

        let side = if is_same_all {
            puzzle.side(p)
        } else {
            None
        };
        let is_h = eh_p == Some(Edge::Line) && eh_l == Some(Edge::Line);
        let is_v = ev_p == Some(Edge::Line) && ev_u == Some(Edge::Line);

        if is_same_all {
            try!(printer.write_pretty(side, "."));
        } else if is_h {
            try!(printer.write_pretty(side, "-"));
        } else if is_v {
            try!(printer.write_pretty(side, "|"));
        } else {
            try!(printer.write_pretty(side, "+"));
        }
        Ok(())
    }
}

struct EdgeH {
    str_cross: String,
    str_line: String,
    str_unknown: String,
}

impl EdgeH {
    fn new(conf: &Config) -> EdgeH {
        EdgeH {
            str_cross: iter::repeat(' ').take(conf.cell_width).collect(),
            str_line: iter::repeat('-').take(conf.cell_width).collect(),
            str_unknown: iter::repeat('~').take(conf.cell_width).collect(),
        }
    }

    fn pprint<P>(&self, printer: &mut P, puzzle: &Puzzle, p: Point) -> io::Result<()>
        where P: Printer
    {
        let (s, side) = match puzzle.edge_h(p) {
            Some(Edge::Cross) => (&self.str_cross, puzzle.side(p)),
            Some(Edge::Line) => (&self.str_line, None),
            None => (&self.str_unknown, None),
        };
        try!(printer.write_pretty(side, s));
        Ok(())
    }
}

struct Label {
    width: usize,
    order: i32,
    space: String,
}

impl Label {
    fn new(conf: &Config) -> Label {
        Label {
            width: conf.cell_width,
            order: 10i32.pow(conf.cell_width as u32),
            space: format!("{:1$}", "", conf.cell_width),
        }
    }

    fn pprint<P>(&self, printer: &mut P, n: i32, num_line: bool) -> io::Result<()>
        where P: Printer
    {
        if num_line {
            try!(printer.write_plain(&format!("{:^1$}", n % self.order, self.width)));
        } else {
            try!(printer.write_plain(&self.space));
        }
        Ok(())
    }
}

struct EdgeV;

impl EdgeV {
    fn new(_conf: &Config) -> EdgeV {
        EdgeV
    }

    fn pprint<P>(&self, printer: &mut P, puzzle: &Puzzle, p: Point) -> io::Result<()>
        where P: Printer
    {
        let (s, side) = match puzzle.edge_v(p) {
            Some(Edge::Cross) => (" ", puzzle.side(p)),
            Some(Edge::Line) => ("|", None),
            None => ("?", None),
        };
        try!(printer.write_pretty(side, s));
        Ok(())
    }
}

struct Cell {
    nums: [String; 5],
    space: String,
}

impl Cell {
    fn new(conf: &Config) -> Cell {
        Cell {
            nums: [format!("{:^1$}", 0, conf.cell_width),
                   format!("{:^1$}", 1, conf.cell_width),
                   format!("{:^1$}", 2, conf.cell_width),
                   format!("{:^1$}", 3, conf.cell_width),
                   format!("{:^1$}", 4, conf.cell_width)],
            space: format!("{:^1$}", "", conf.cell_width),
        }
    }

    fn pprint<P>(&self,
                 printer: &mut P,
                 puzzle: &Puzzle,
                 p: Point,
                 num_line: bool)
                 -> io::Result<()>
        where P: Printer
    {
        let side = puzzle.side(p);
        match puzzle.hint(p) {
            Some(x) if num_line => try!(printer.write_pretty(side, &self.nums[x as usize])),
            _ => try!(printer.write_pretty(side, &self.space)),
        }
        Ok(())
    }
}

pub fn print(conf: &Config, puzzle: &Puzzle) -> io::Result<()> {
    let is_color = conf.mode == Mode::Color;

    let table = Table::new(conf);
    match StdoutPrinter::new(is_color) {
        StdoutPrinter::Pretty(mut p) => table.pprint(&mut p, puzzle),
        StdoutPrinter::Raw(mut p) => table.pprint(&mut p, puzzle),
    }
}
