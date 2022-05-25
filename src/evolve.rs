use std::{collections::{HashMap, VecDeque}, hash::BuildHasherDefault, mem::replace};

use fxhash::FxHasher64;

use crate::{tiles::{UniqueSlices, Possibility, ID}, rules::Rules};

const OFFSET: [(i32, i32); 8] = [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)];

/// Evolver can evolve the state forward.
/// It cannot backtrack on its own if something goes wrong.
#[derive(Debug, Clone)]
pub struct Evolver {
    by_loc: Snapshot,
    possibilities: UniqueSlices<Possibility>,
    weights: UniqueSlices<u32>,
    rules: Rules,
    possible_buf: Vec<Possibility>,
    score_buf: Vec<u32>,
    queue: VecDeque<(i32, i32)>
}
impl Evolver {
    pub fn new(rules: Rules) -> Evolver {
        let (possible_buf, score_buf) = rules.possible();

        let mut possibilities = UniqueSlices::new();
        let mut weights = UniqueSlices::new();
        possibilities.identify(&possible_buf);
        possibilities.identify(&[]);
        weights.identify(&score_buf);
        weights.identify(&[]);

        Evolver {
            by_loc: Snapshot::default(),
            possibilities,
            weights,
            rules,
            possible_buf,
            score_buf,
            queue: VecDeque::new()
        }
    }
    fn update_tile(&mut self, x: i32, y: i32) -> Result<bool, ()> {
        let surroundings = OFFSET.map(|(xb, yb)| {
            let x = x + xb;
            let y = y + yb;

            let id = match self.by_loc.get(&Position::new(x, y)) {
                Some((id, _)) => id.clone(),
                None => ID::ZERO
            };
            self.possibilities.get(id)
        });
        if let Some((poss_id, score_id)) = self.by_loc.get_mut(&Position::new(x, y)) {
            let center = self.possibilities.get(poss_id.clone());

            self.rules.check(center, &surroundings, &mut self.possible_buf, &mut self.score_buf);
            let new_poss_id = self.possibilities.identify(&self.possible_buf);
            let new_score_id = self.weights.identify(&self.score_buf);

            let change = new_poss_id != *poss_id || (new_score_id != *score_id && *score_id == ID::ONE);
            if change {
                if new_poss_id == ID::ONE {
                    return Err(())
                }

                *poss_id = new_poss_id;
                *score_id = self.weights.identify(&self.score_buf);
            }

            Ok(change)
        } else {
            let center = self.possibilities.get(ID::ZERO);

            self.rules.check(center, &surroundings, &mut self.possible_buf, &mut self.score_buf);
            let new_poss_id = self.possibilities.identify(&self.possible_buf);
            let new_score_id = self.weights.identify(&self.score_buf);

            let change = new_poss_id != ID::ZERO || new_score_id != ID::ZERO;
            if change {
                if new_poss_id == ID::ONE {
                    return Err(());
                }
                self.by_loc.insert(Position::new(x, y), (new_poss_id, new_score_id));
            }
            
            Ok(change)
        }
    }
    fn propagate_around(&mut self, x: i32, y: i32) -> Result<(), ()> {
        self.queue.extend(OFFSET.iter().map(|(xb, yb)| (x + xb, y + yb)));
        while let Some((x, y)) = self.queue.pop_front() {
            let changed = self.update_tile(x, y)?;
            if changed {
                let affected = OFFSET.iter().map(|(xb, yb)| (x + xb, y + yb));
                self.queue.extend(affected);
            }
        }
        Ok(())
    }
    pub fn snapshot(&self) -> Snapshot {
        self.by_loc.clone()
    }
    pub fn restore(&mut self, prev_snap: Snapshot) {
        let _ = replace(&mut self.by_loc, prev_snap);
    }
    pub fn collapse(&mut self, x: i32, y: i32, f: impl FnOnce(&[Possibility], &[u32]) -> Option<Possibility>) -> Result<Possibility, CollapseErr<Possibility>> {
        match self.by_loc.get_mut(&Position::new(x, y)) {
            Some((poss_id, weight_id)) => {
                let p = self.possibilities.get(poss_id.clone());
                if p.len() <= 1 {
                    return match p.get(0) {
                        Some(p) => Ok(p.clone()),
                        None => Err(CollapseErr::Collapse)
                    };
                }
                let s = self.weights.get(weight_id.clone());
                let collapse_value = f(p, s).ok_or(CollapseErr::Collapse)?;
                *poss_id = self.possibilities.identify(&[collapse_value]);
                *weight_id = ID::ONE;
                self.propagate_around(x, y).map_err(|_| CollapseErr::Propagate(collapse_value))?;
                Ok(collapse_value)
            },
            None => {
                let p = self.possibilities.get(ID::ZERO);
                let s = self.weights.get(ID::ZERO);
                let collapse_value = f(p, s).ok_or(CollapseErr::Collapse)?;
                let poss_id = self.possibilities.identify(&[collapse_value]);
                self.by_loc.insert(Position::new(x, y), (poss_id, ID::ONE));
                self.propagate_around(x, y).map_err(|_| CollapseErr::Propagate(collapse_value))?;
                Ok(collapse_value)
            }
        }
    }
    pub fn ensure_impossible(&mut self, x: i32, y: i32, impossible: Possibility) -> Result<(), CollapseErr<()>> {
        match self.by_loc.get_mut(&Position::new(x, y)) {
            Some((poss_id, _)) => {
                let mut full_possibilities = self.possibilities.get(poss_id.clone()).to_owned();
                match full_possibilities.binary_search(&impossible) {
                    Ok(i) => {full_possibilities.remove(i);},
                    Err(_) => return Ok(())
                }
                if full_possibilities.len() == 0 {
                    return Err(CollapseErr::Collapse);
                }
                let new_id = self.possibilities.identify(&full_possibilities);
                *poss_id = new_id;
                self.update_tile(x, y).map_err(|_| CollapseErr::Collapse)?;
                self.propagate_around(x, y).map_err(|_| CollapseErr::Propagate(()))?;
                Ok(())
            },
            None => {
                let mut full_possibilities = self.possibilities.get(ID::ZERO).to_owned();
                match full_possibilities.binary_search(&impossible) {
                    Ok(i) => {full_possibilities.remove(i);},
                    Err(_) => return Ok(())
                }
                if full_possibilities.len() == 0 {
                    return Err(CollapseErr::Collapse);
                }
                let new_id = self.possibilities.identify(&full_possibilities);
                self.by_loc.insert(Position::new(x, y), (new_id, ID::ZERO));
                self.update_tile(x, y).map_err(|_| CollapseErr::Collapse)?;
                self.propagate_around(x, y).map_err(|_| CollapseErr::Propagate(()))?;
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum CollapseErr<T> {
    Collapse,
    Propagate(T)
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Position(u64);
impl Position {
    fn new(x: i32, y: i32) -> Position {
        let a = (x as u64) << 32;
        Position(a & (y as u64))
    }
}
pub type Snapshot = HashMap<Position, (ID<Possibility>, ID<u32>), BuildHasherDefault<FxHasher64>>;
