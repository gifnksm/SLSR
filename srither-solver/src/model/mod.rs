// Copyright (c) 2016 srither-solver developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use {Error, SolverResult};
pub use self::connect_map::ConnectMap;
pub use self::side_map::SideMap;

mod connect_map;
pub mod pattern;
mod side_map;
pub mod theorem;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum State<T> {
    Fixed(T),
    Unknown,
    Conflict,
}

impl<T> Into<SolverResult<Option<T>>> for State<T> {
    fn into(self) -> SolverResult<Option<T>> {
        match self {
            State::Fixed(st) => Ok(Some(st)),
            State::Unknown => Ok(None),
            State::Conflict => Err(Error::invalid_board()),
        }
    }
}
