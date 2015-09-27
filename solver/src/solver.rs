use slsr_core::puzzle::{Puzzle, Side};
use slsr_core::geom::{CellId, Geom};

use model::connect_map::ConnectMap;
use model::side_map::SideMap;
use model::theorem::Theorem;
use step::apply_theorem::TheoremPool;
use ::{SolverResult, State, LogicError};

#[derive(Clone, Debug)]
pub struct Solver {
    theorem_pool: TheoremPool,
    side_map: SideMap,
    connect_map: Option<ConnectMap>
}

impl Solver {
    pub fn new<I>(puzzle: &Puzzle, theorem: I) -> SolverResult<Solver>
        where I: Iterator<Item=Theorem>
    {
        let mut side_map = SideMap::from(puzzle);
        Ok(Solver {
            theorem_pool: try!(TheoremPool::new(theorem, &mut side_map)),
            side_map: side_map,
            connect_map: None
        })
    }

    pub fn revision(&self) -> u32 {
        self.side_map.revision()
    }
    pub fn all_filled(&self) -> bool {
        self.side_map.all_filled()
    }

    pub fn get_side(&mut self, p: CellId) -> State<Side> {
        self.side_map.get_side(p)
    }
    pub fn set_inside(&mut self, p: CellId) -> bool {
        self.side_map.set_inside(p)
    }
    pub fn set_outside(&mut self, p: CellId) -> bool {
        self.side_map.set_outside(p)
    }

    pub fn validate_result(&mut self) -> SolverResult<()> {
        try!(self.sync_connection());
        if self.connect_map().count_area() != 2 {
            return Err(LogicError)
        }
        Ok(())
    }

    pub fn get_unknown_points(&mut self) -> Vec<CellId> {
        let mut pts = vec![];

        let mut conn_map = self.connect_map();

        for i in 0..conn_map.cell_len() {
            let p = CellId::new(i);
            let a = conn_map.get(p);
            if a.coord() == p && a.side() == State::Unknown {
                pts.push((p, a.unknown_edge().len()));
            }
        }

        pts.sort_by(|a, b| a.1.cmp(&b.1));
        pts.into_iter().map(|pair| pair.0).collect()
    }

    pub fn apply_all_theorem(&mut self) -> SolverResult<()> {
        self.theorem_pool.apply_all(&mut self.side_map)
    }
    pub fn connect_analysis(&mut self) -> SolverResult<()> {
        self.create_connect_map();
        ::step::connect_analysis::run(&mut self.side_map,
                                      self.connect_map.as_mut().unwrap())
    }

    fn create_connect_map(&mut self) {
        if self.connect_map.is_none() {
            self.connect_map = Some(ConnectMap::from(&mut self.side_map));
        }
    }
    fn connect_map(&mut self) -> &mut ConnectMap {
        self.create_connect_map();
        self.connect_map.as_mut().unwrap()
    }
    fn sync_connection(&mut self) -> SolverResult<()> {
        self.create_connect_map();
        self.connect_map.as_mut().unwrap().sync(&mut self.side_map)
    }
}

impl Into<Result<Puzzle, LogicError>> for Solver {
    fn into(self) -> Result<Puzzle, LogicError> {
        self.side_map.into()
    }
}
