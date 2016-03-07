// Copyright (c) 2016 srither-solver developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::iter::FromIterator;

use srither_core::puzzle::Puzzle;
use srither_core::geom::{CellId, Geom, Move, Point, Rotation, Size};

use SolverResult;
use model::SideMap;
use model::pattern::{EdgePattern, HintPattern, MatchResult as PatternMatchResult};

mod parse;

#[derive(Clone, Debug)]
pub enum MatchResult {
    Complete(Vec<EdgePattern<CellId>>),
    Partial(PartialTheorem),
    Conflict,
}

impl MatchResult {
    pub fn update(self, side_map: &mut SideMap, new_theorem: &mut Vec<PartialTheorem>) {
        match self {
            MatchResult::Complete(result) => {
                for pat in &result {
                    pat.apply(side_map);
                }
            }
            MatchResult::Partial(theo) => new_theorem.push(theo),
            MatchResult::Conflict => {}
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Theorem {
    size: Size,
    hint_matcher: Vec<HintPattern>,
    edge_matcher: Vec<EdgePattern<Point>>,
    result: Vec<EdgePattern<Point>>,
    closed_hint: Option<(u32, Vec<HintPattern>)>,
}

impl Theorem {
    fn rotate(&self, rot: Rotation) -> Theorem {
        let mv = rot * Move(self.size.0, self.size.1);
        let mut d = Move(0, 0);
        if mv.0 < 0 {
            d = d + Move(-mv.0 - 1, 0);
        }
        if mv.1 < 0 {
            d = d + Move(0, -mv.1 - 1);
        }
        let size = Size(mv.0.abs(), mv.1.abs());

        let mut hint_matcher = Vec::from_iter(self.hint_matcher
                                                  .iter()
                                                  .map(|x| x.rotate(rot).shift(d)));
        hint_matcher.sort();
        hint_matcher.dedup();

        let mut edge_matcher = Vec::from_iter(self.edge_matcher
                                                  .iter()
                                                  .map(|x| x.rotate(rot).shift(d)));
        edge_matcher.sort();
        edge_matcher.dedup();

        let mut result = Vec::from_iter(self.result.iter().map(|x| x.rotate(rot).shift(d)));
        result.sort();
        result.dedup();

        let closed_hint = self.closed_hint.as_ref().map(|&(sum, ref pat)| {
            let mut pat = Vec::from_iter(pat.iter().map(|x| x.rotate(rot).shift(d)));
            pat.sort();
            pat.dedup();
            (sum, pat)
        });

        Theorem {
            size: size,
            hint_matcher: hint_matcher,
            edge_matcher: edge_matcher,
            result: result,
            closed_hint: closed_hint,
        }
    }

    pub fn all_rotations(self) -> Vec<Theorem> {
        let deg90 = self.rotate(Rotation::CCW90);
        let deg180 = self.rotate(Rotation::CCW180);
        let deg270 = self.rotate(Rotation::CCW270);
        let h_deg0 = self.rotate(Rotation::H_FLIP);
        let h_deg90 = h_deg0.rotate(Rotation::CCW90);
        let h_deg180 = h_deg0.rotate(Rotation::CCW180);
        let h_deg270 = h_deg0.rotate(Rotation::CCW270);
        let mut rots = vec![self, deg90, deg180, deg270, h_deg0, h_deg90, h_deg180, h_deg270];

        rots.sort();
        rots.dedup();

        rots
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn head(&self) -> Option<HintPattern> {
        self.hint_matcher.get(0).cloned()
    }

    fn can_close(shift: Move,
                 puzzle: &Puzzle,
                 sum_of_hint: u32,
                 hpat: &[HintPattern],
                 sum_of_hpat: u32)
                 -> bool {
        if sum_of_hint > sum_of_hpat {
            return false;
        }

        let mut ava_sum = 0;
        for h in hpat {
            if let Some(n) = puzzle.hint(h.point() + shift) {
                if n != h.hint() {
                    return false;
                }
                ava_sum += n as u32;
            }
        }

        if ava_sum != sum_of_hint {
            return false;
        }

        true
    }

    pub fn shift_matches(&self,
                         shift: Move,
                         puzzle: &Puzzle,
                         sum_of_hint: u32,
                         side_map: &mut SideMap)
                         -> SolverResult<MatchResult> {
        let mut num_matcher = 0;
        for matcher in &self.hint_matcher {
            match try!(matcher.shift(shift).matches::<Point>(puzzle)) {
                PatternMatchResult::Complete => {}
                PatternMatchResult::Conflict => {
                    return Ok(MatchResult::Conflict);
                }
                PatternMatchResult::Partial(_) => panic!(),
            }
        }

        for matcher in &self.edge_matcher {
            match try!(matcher.shift(shift).matches(puzzle, side_map)) {
                PatternMatchResult::Complete => {}
                PatternMatchResult::Partial(_) => {
                    num_matcher += 1;
                }
                PatternMatchResult::Conflict => {
                    return Ok(MatchResult::Conflict);
                }
            }
        }

        if let Some((sum_of_hpat, ref hpat)) = self.closed_hint {
            if Theorem::can_close(shift, puzzle, sum_of_hint, hpat, sum_of_hpat) {
                return Ok(MatchResult::Conflict);
            }
        }

        let result = self.result
                         .iter()
                         .map(|pat| pat.shift(shift).to_cellid(&puzzle))
                         .collect();

        if num_matcher == 0 {
            return Ok(MatchResult::Complete(result));
        }

        let mut new_matcher = Vec::with_capacity(num_matcher);
        for matcher in &self.edge_matcher {
            match try!(matcher.shift(shift).matches(puzzle, side_map)) {
                PatternMatchResult::Complete => {}
                PatternMatchResult::Partial(m) => {
                    new_matcher.push(m);
                }
                PatternMatchResult::Conflict => panic!(),
            }
        }

        Ok(MatchResult::Partial(PartialTheorem {
            matcher: new_matcher,
            result: result,
        }))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PartialTheorem {
    matcher: Vec<EdgePattern<CellId>>,
    result: Vec<EdgePattern<CellId>>,
}

impl PartialTheorem {
    pub fn dummy() -> PartialTheorem {
        PartialTheorem {
            matcher: vec![],
            result: vec![],
        }
    }

    pub fn merge(&mut self, other: &PartialTheorem) -> Result<(), ()> {
        if self.matcher != other.matcher {
            return Err(());
        }

        self.result.extend(other.result.iter().cloned());
        self.result.sort();
        self.result.dedup();
        Ok(())
    }

    pub fn matches(mut self, side_map: &mut SideMap) -> SolverResult<MatchResult> {
        unsafe {
            // Assume the elements of self.matcher is copyable.
            let len = self.matcher.len();
            let p = self.matcher.as_mut_ptr();
            let mut w = 0;
            for r in 0..len {
                let read = *p.offset(r as isize);

                match try!(read.matches(side_map)) {
                    PatternMatchResult::Complete => {}
                    PatternMatchResult::Partial(e) => {
                        *p.offset(w as isize) = e;
                        w += 1;
                    }
                    PatternMatchResult::Conflict => {
                        return Ok(MatchResult::Conflict);
                    }
                }
            }
            self.matcher.set_len(w);
        }

        let m = if self.matcher.is_empty() {
            MatchResult::Complete(self.result)
        } else {
            MatchResult::Partial(self)
        };
        Ok(m)
    }

    pub fn num_matcher(&self) -> usize {
        self.matcher.len()
    }

    pub fn matcher_edges(&self) -> &[EdgePattern<CellId>] {
        &self.matcher
    }

    pub fn result_edges(self) -> Vec<EdgePattern<CellId>> {
        self.result
    }
}

#[cfg(test)]
mod tests {
    use srither_core::geom::Rotation;
    use super::Theorem;

    #[test]
    fn rotate() {
        let deg0 = r"
+ + + ! + + +
   a  !  bxa
+ + + ! +x+-+
 a 3  !  a|3
+ + + ! + + +
      !    B
+ + + ! + + +
"
                       .parse::<Theorem>()
                       .unwrap();

        let deg90 = r"
+ + + + ! + + + +
 a 3    !  a|3 B
+ + + + ! +x+-+ +
   a    !  bxa
+ + + + ! + + + +
"
                        .parse::<Theorem>()
                        .unwrap();

        let deg180 = r"
+ + + ! + + +
      !  B
+ + + ! + + +
 3 a  !  3|a
+ + + ! +-+x+
 a    !  axb
+ + + ! + + +
"
                         .parse::<Theorem>()
                         .unwrap();

        let deg270 = r"
+ + + + ! + + + +
   a    !    axb
+ + + + ! + +-+x+
   3 a  !  B 3|a
+ + + + ! + + + +
"
                         .parse::<Theorem>()
                         .unwrap();

        let h_flip = r"
+ + + ! + + +
 a    !  axb
+ + + ! +-+x+
 3 a  !  3|a
+ + + ! + + +
      !  B
+ + + ! + + +
"
                         .parse::<Theorem>()
                         .unwrap();

        let v_flip = r"
+ + + ! + + +
      !    B
+ + + ! + + +
 a 3  !  a|3
+ + + ! +x+-+
   a  !  bxa
+ + + ! + + +
"
                         .parse::<Theorem>()
                         .unwrap();

        assert_eq!(deg0.clone(), deg0.clone().rotate(Rotation::CCW0));
        assert_eq!(deg90.clone(), deg0.clone().rotate(Rotation::CCW90));
        assert_eq!(deg180.clone(), deg0.clone().rotate(Rotation::CCW180));
        assert_eq!(deg270.clone(), deg0.clone().rotate(Rotation::CCW270));
        assert_eq!(h_flip.clone(), deg0.clone().rotate(Rotation::H_FLIP));
        assert_eq!(v_flip.clone(), deg0.clone().rotate(Rotation::V_FLIP));
        assert_eq!(v_flip.clone(), h_flip.clone().rotate(Rotation::CCW180));

        let mut rots = &mut [deg0.clone(),
                             deg90,
                             deg180,
                             deg270,
                             h_flip.clone(),
                             h_flip.clone().rotate(Rotation::CCW90),
                             h_flip.clone().rotate(Rotation::CCW180),
                             h_flip.clone().rotate(Rotation::CCW270)];
        rots.sort();
        assert_eq!(rots, &deg0.all_rotations()[..]);
    }

    #[test]
    fn all_rotations() {
        let theo = r"
+ + ! +x+
 0  ! x0x
+ + ! +x+
"
                       .parse::<Theorem>()
                       .unwrap();
        let rots = theo.clone().all_rotations();
        assert_eq!(&[theo], &rots[..]);
    }
}
