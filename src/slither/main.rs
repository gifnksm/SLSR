#![warn(bad_style,
        unused, unused_extern_crates, unused_import_braces,
        unused_qualifications, unused_results, unused_typecasts)]
#![allow(unstable)]

extern crate "union-find" as union_find;
extern crate term;

use std::io::stdio;
use board::Board;

mod board;
mod connect_map;
mod geom;
mod pprint;
mod side_map;
mod solver;

fn main() {
    let board = Board::from_reader(stdio::stdin()).unwrap();
    let board = solver::solve(&board).unwrap();
    let _ = pprint::print(&board);
}
