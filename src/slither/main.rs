#![feature(core)]
#![feature(collections)]
#![feature(libc)]
#![feature(plugin)]
#![feature(unicode)]
#![warn(bad_style,
        unused, unused_extern_crates, unused_import_braces,
        unused_qualifications, unused_results, unused_typecasts)]

#![plugin(docopt_macros)]

extern crate docopt;
extern crate libc;
extern crate rustc_serialize;
extern crate union_find;
extern crate term;

use std::default::Default;
use std::io;
use std::io::prelude::*;
use board::Board;
use locale::Category;

mod board;
mod geom;
mod locale;
mod pprint;
mod solver;
mod util;

docopt! {
    Args derive Debug, "
Usage: slither [options]
       slither --help

Options:
  -h, --help       Show this message.
  --width WIDTH    Specify cell width.
  --height HEIGHT  Specify cell height.
  --mode MODE      Specify pretty print mode.
                   Valid values: ascii, unicode.
  --cjk CJK        Specify pretty print char width.
                   Valid values: auto, yes, no.
",
    flag_width: Option<Width>,
    flag_height: Option<Height>,
    flag_mode: Option<pprint::Mode>,
    flag_cjk: Option<YesNo>
}

#[derive(Debug)]
struct Width(usize);
impl rustc_serialize::Decodable for Width {
    fn decode<D: rustc_serialize::Decoder>(d: &mut D) -> Result<Width, D::Error> {
        let w = try!(d.read_usize());
        if w == 0 {
            Err(d.error(&format!("Could not decode '{}' as width.", w)))
        } else {
            Ok(Width(w))
        }
    }
}
impl Default for Width {
    fn default() -> Width { Width(2) }
}

#[derive(Debug)]
struct Height(usize);
impl rustc_serialize::Decodable for Height {
    fn decode<D: rustc_serialize::Decoder>(d: &mut D) -> Result<Height, D::Error> {
        let h = try!(d.read_usize());
        if h == 0 {
            Err(d.error(&format!("Could not decode '{}' as height.", h)))
        } else {
            Ok(Height(h))
        }
    }
}
impl Default for Height {
    fn default() -> Height { Height(1) }
}

#[derive(Copy, Debug, RustcDecodable, Eq, PartialEq)]
pub enum YesNo {
    Auto, Yes, No,
}
impl Default for YesNo {
    fn default() -> YesNo { YesNo::Auto }
}

fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());

    let mut input = String::new();
    let _ = io::stdin().read_to_string(&mut input).unwrap();
    let board = input.parse::<Board>().unwrap();
    let board = solver::solve(&board).unwrap();

    if pprint::is_pprintable() {
        let is_cjk = match args.flag_cjk.unwrap_or_default() {
            YesNo::Yes => true,
            YesNo::No => false,
            YesNo::Auto => {
                let loc = locale::setlocale(Category::CType, "");
                loc.starts_with("ja") || loc.starts_with("ko") || loc.starts_with("zh")
            }
        };

        let conf = pprint::Config {
            mode: args.flag_mode.unwrap_or_default(),
            is_cjk: is_cjk,
            cell_width: args.flag_width.unwrap_or_default().0,
            cell_height: args.flag_height.unwrap_or_default().0
        };
        let _ = pprint::print(&conf, &board);
    } else {
        print!("{}", board.to_string());
    }
}
