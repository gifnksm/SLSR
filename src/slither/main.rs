#![warn(bad_style,
        unused, unused_extern_crates, unused_import_braces,
        unused_qualifications, unused_results, unused_typecasts)]
#![allow(unstable)]

extern crate "union-find" as union_find;
extern crate term;

use std::io::stdio;
use board::Board;

mod board;
mod geom;
mod pprint;
mod solver;

fn main() {
    let raw_input = stdio::stdin().read_to_end().unwrap();
    let input = String::from_utf8(raw_input).unwrap();
    let board = input.parse::<Board>().unwrap();
    let board = solver::solve(&board).unwrap();
    let _ = pprint::print(&board);
}
