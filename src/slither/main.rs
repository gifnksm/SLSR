#![feature(plugin)]
#![warn(bad_style,
        unused, unused_extern_crates, unused_import_braces,
        unused_qualifications, unused_results, unused_typecasts)]
#![allow(unstable)]

extern crate docopt;
#[plugin] #[no_link] extern crate docopt_macros;
extern crate libc;
extern crate "rustc-serialize" as rustc_serialize;
extern crate "union-find" as union_find;
extern crate term;

use std::io::stdio;
use board::Board;

mod board;
mod geom;
mod locale;
mod pprint;
mod solver;

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
    flag_cjk: Option<pprint::Type>
}

#[derive(Debug)]
struct Width(usize);
impl rustc_serialize::Decodable for Width {
    fn decode<D: rustc_serialize::Decoder>(d: &mut D) -> Result<Width, D::Error> {
        let w = try!(d.read_usize());
        if w == 0 {
            Err(d.error(&format!("Could not decode '{}' as width.", w)[]))
        } else {
            Ok(Width(w))
        }
    }
}

#[derive(Debug)]
struct Height(usize);
impl rustc_serialize::Decodable for Height {
    fn decode<D: rustc_serialize::Decoder>(d: &mut D) -> Result<Height, D::Error> {
        let h = try!(d.read_usize());
        if h == 0 {
            Err(d.error(&format!("Could not decode '{}' as height.", h)[]))
        } else {
            Ok(Height(h))
        }
    }
}

fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());

    let raw_input = stdio::stdin().read_to_end().unwrap();
    let input = String::from_utf8(raw_input).unwrap();
    let board = input.parse::<Board>().unwrap();
    let board = solver::solve(&board).unwrap();

    if stdio::stdout_raw().isatty() {
        let conf = pprint::Config {
            mode: args.flag_mode.unwrap_or(pprint::Mode::Unicode),
            cjk: args.flag_cjk.unwrap_or(pprint::Type::Auto),
            cell_width: args.flag_width.unwrap_or(Width(2)).0,
            cell_height: args.flag_height.unwrap_or(Height(1)).0
        };
        let _ = pprint::print(&conf, &board);
    } else {
        print!("{}", board.to_string());
    }
}
