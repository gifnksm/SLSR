use std::mem;
use slsr_core::geom::{Geom, Move, Table};
use slsr_core::puzzle::Hint;

use ::SolverResult;
use ::model::side_map::SideMap;
use ::model::theorem::{Pattern, Theorem, TheoremMatcher};

#[derive(Clone, Debug)]
pub struct TheoremPool {
    data: Vec<TheoremMatcher>
}

impl TheoremPool {
    pub fn new<'a, T>(theo_defs: T,
                      hint: &Table<Hint>,
                      sum_of_hint: u32,
                      side_map: &mut SideMap)
                      -> SolverResult<TheoremPool>
        where T: IntoIterator<Item=Theorem>
    {
        let it = theo_defs
            .into_iter()
            .flat_map(|theo| theo.all_rotations());

        let mut hint_theorem = [vec![], vec![], vec![], vec![], vec![]];
        let mut nonhint_theorem = vec![];

        for theo in it {
            match theo.head() {
                Pattern::Hint(h) => hint_theorem[h.hint() as usize].push(theo),
                _ => nonhint_theorem.push(theo)
            }
        }

        let mut data = vec![];

        for p in hint.points() {
            if let Some(x) = hint[p] {
                for theo in &hint_theorem[x as usize] {
                    let o = match theo.head() {
                        Pattern::Hint(hint) => hint.point(),
                        _ => panic!()
                    };
                    let matcher = theo.clone().shift(p - o);
                    try!(matcher.matches(hint, sum_of_hint, side_map))
                        .update(side_map, &mut data);
                }
            }
        }

        for theo in nonhint_theorem {
            let sz = theo.size();
            for r in (1 - sz.0)..(hint.row() + sz.0 - 1) {
                for c in (1 - sz.1)..(hint.column() + sz.1 - 1) {
                    let matcher = theo.clone().shift(Move(r, c));
                    try!(matcher.matches(hint, sum_of_hint, side_map))
                        .update(side_map, &mut data);
                }
            }
        }

        let mut pool = TheoremPool { data: data };
        pool.merge_dup();

        Ok(pool)
    }

    pub fn apply_all(&mut self, side_map: &mut SideMap) -> SolverResult<()> {
        let cap = self.data.len();

        for matcher in mem::replace(&mut self.data, Vec::with_capacity(cap)) {
            try!(matcher.matches(side_map))
                .update(side_map, &mut self.data);
        }

        Ok(())
    }

    fn merge_dup(&mut self) {
        self.data.sort();
        // Merge elements that have same matchers.
        unsafe {
            let len = self.data.len();
            let p = self.data.as_mut_ptr();

            let mut w = 1;
            for r in 1..len {
                let read = p.offset(r as isize);
                let cmp = p.offset((w - 1) as isize);

                match (*cmp).merge(&*read) {
                    Ok(()) => {}
                    Err(()) => {
                        let write = cmp.offset(1);
                        mem::swap(&mut *write, &mut *read);
                        w += 1;
                    }
                }
            }

            self.data.truncate(w);
        }
    }
}
