#![warn(bad_style)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
// #![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unused_results)]

#![feature(plugin)]
#![plugin(docopt_macros)]

extern crate docopt;
extern crate libc;
extern crate rustc_serialize;
extern crate term;
extern crate slsr_core;
extern crate slsr_solver;

use std::default::Default;
use std::io;
use std::io::prelude::*;
use slsr_core::board::Board;

mod pprint;

docopt! {
    Args derive Debug, "
Usage: slither [options]
       slither --help

Options:
  -h, --help       Show this message.
  --width WIDTH    Specify cell width.
  --height HEIGHT  Specify cell height.
",
    flag_width: Option<Width>,
    flag_height: Option<Height>
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

fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());

    let mut input = String::new();
    let _ = io::stdin().read_to_string(&mut input).unwrap();
    let board = input.parse::<Board>().unwrap();
    let board = slsr_solver::solve(&board).unwrap();

    if pprint::is_pprintable() {
        let conf = pprint::Config {
            cell_width: args.flag_width.unwrap_or_default().0,
            cell_height: args.flag_height.unwrap_or_default().0
        };
        let _ = pprint::print(&conf, &board);
    } else {
        print!("{}", board.to_string());
    }
}
