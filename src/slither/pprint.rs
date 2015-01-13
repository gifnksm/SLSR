use std::fmt;
use std::io::IoResult;
use std::io::stdio::{self, StdWriter};
use geom::{Geom, Point, UP, LEFT};
use hint::Hint;
use side_map::{SideMap, Relation, Side};
use term::{self, Terminal, WriterWrapper};
use term::color::{self, Color};

enum Output<T> {
    Pretty(Box<Terminal<WriterWrapper> + Send>),
    Raw(T)
}

fn type_to_color(ty: Side) -> (Color, Color) {
    match ty {
        Side::In       => (color::BRIGHT_YELLOW, color::BLACK),
        Side::Out      => (color::BLACK,         color::BRIGHT_WHITE),
        Side::Unknown  => (color::BRIGHT_WHITE,  color::BLACK),
        Side::Conflict => (color::RED,           color::BRIGHT_WHITE),
    }
}

struct Printer<'a> {
    output: Output<StdWriter>,
    side_map: &'a mut SideMap,
    hint: &'a Hint
}

impl<'a> Printer<'a> {
    fn new(side_map: &'a mut SideMap, hint: &'a Hint) -> Printer<'a> {
        let output = if stdio::stdout_raw().isatty() {
            match term::stdout() {
                Some(t) => Output::Pretty(t),
                None    => Output::Raw(stdio::stdout_raw())
            }
        } else {
            Output::Raw(stdio::stdout_raw())
        };
        Printer {
            output: output,
            side_map: side_map,
            hint: hint
        }
    }

    fn write_pretty(&mut self, ty: Side, s: &str) -> IoResult<()> {
        match self.output {
            Output::Pretty(ref mut term) => {
                let (bg, fg) = type_to_color(ty);
                try!(term.fg(fg));
                try!(term.bg(bg));
                try!(term.write(s.as_bytes()));
                try!(term.reset());
                Ok(())
            }
            Output::Raw(ref mut stdout) => stdout.write(s.as_bytes())
        }
    }
    fn write_pretty_fmt(&mut self, ty: Side, fmt: fmt::Arguments) -> IoResult<()> {
        match self.output {
            Output::Pretty(ref mut term) => {
                let (bg, fg) = type_to_color(ty);
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
        let row = self.side_map.row();
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
        for x in 0 .. self.side_map.column() {
            try!(self.write_plain_fmt(format_args!(" {:2}", x)));
        }
        try!(self.write_plain("\n"));
        Ok(())
    }

    fn edge_row(&mut self, y: i32) -> IoResult<()> {
        let col = self.side_map.column();
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
        let col = self.side_map.column();
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
        let ps = &[p, p + UP, p + LEFT, p + UP + LEFT];
        let ty = if self.side_map.is_same_all(ps) {
            self.side_map.get_side(p)
        } else {
            Side::Unknown
        };
        try!(self.write_pretty(ty, "+"));
        Ok(())
    }

    fn edge_h(&mut self, p: Point) -> IoResult<()> {
        let ty = if self.side_map.is_same(p, p + UP) {
            self.side_map.get_side(p)
        } else {
            Side::Unknown
        };
        let s = match self.side_map.get_relation(p, p + UP) {
            Relation::Same      => " ",
            Relation::Different => "-",
            Relation::Unknown   => "~",
            Relation::Conflict  => "!"
        };
        try!(self.write_pretty_fmt(ty, format_args!("{}{}", s, s)));
        Ok(())
    }

    fn edge_v(&mut self, p: Point) -> IoResult<()> {
        let ty = if self.side_map.is_same(p, p + LEFT) {
            self.side_map.get_side(p)
        } else {
            Side::Unknown
        };
        let s = match self.side_map.get_relation(p, p + LEFT) {
            Relation::Same      => " ",
            Relation::Different => "|",
            Relation::Unknown   => "?",
            Relation::Conflict  => "!"
        };
        try!(self.write_pretty(ty, s));
        Ok(())
    }

    fn cell(&mut self, p: Point) -> IoResult<()> {
        let ty = self.side_map.get_side(p);
        match self.hint[p] {
            Some(x) => try!(self.write_pretty_fmt(ty, format_args!("{} ", x))),
            None    => try!(self.write_pretty(ty, "  "))
        }
        Ok(())
    }
}

pub fn print(side_map: &mut SideMap, hint: &Hint) -> IoResult<()> {
    Printer::new(side_map, hint).print()
}

