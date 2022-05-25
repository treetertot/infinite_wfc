use std::collections::VecDeque;

use rand::{prelude::SmallRng, SeedableRng, Rng};

use crate::{evolve::{Evolver, Snapshot, CollapseErr}, rules::{Rule, Rules}};

#[derive(Debug, Clone)]
pub struct World {
    evolver: Evolver,
    snaps: Snapshots,
    rng: SmallRng
}
impl World {
    pub fn new(rules: &[Rule], snapshot_stability: usize) -> World {
        let rules = Rules::new(rules);
        let evolver = Evolver::new(rules);
        let snaps = Snapshots::new(snapshot_stability);
        World { evolver, snaps, rng: SmallRng::from_entropy() }
    }
    pub fn get(&mut self, x: i32, y: i32) -> u32 {
        let mut snapshot = self.evolver.snapshot();
        let res = loop {
            match self.evolver.collapse(x, y, |possibilities, weights| {
                let total_weight: u32 = weights.iter().sum();
                let mut random_num = self.rng.gen_range(0..total_weight);
                for (i, weight) in weights.iter().enumerate() {
                    if random_num < *weight {
                        return Some(possibilities[i]);
                    }
                    random_num -= *weight;
                }
                None
            }) {
                Ok(p) => break p.0,
                Err(CollapseErr::Collapse) => {
                    snapshot = self.snaps.pop().expect("impossible ruleset");
                    self.evolver.restore(snapshot.clone());
                },
                Err(CollapseErr::Propagate(attempted)) => {
                    self.evolver.restore(snapshot);
                    while self.evolver.ensure_impossible(x, y, attempted).is_err() {
                        snapshot = self.snaps.pop().expect("impossible rules");
                        self.evolver.restore(snapshot);
                    }
                    snapshot = self.evolver.snapshot();
                }
            }
        };
        self.snaps.push(snapshot);
        res
    }
}

#[derive(Debug, Clone)]
struct Snapshots {
    size: usize,
    rings: Vec<SnapRing>,
}
impl Snapshots {
    fn new(size: usize) -> Snapshots {
        Snapshots { size, rings: Vec::new() }
    }
    fn push(&mut self, mut item: Snapshot) {
        let mut iter = self.rings.iter_mut();
        while let Some(ring) = iter.next() {
            item = match ring.push(item) {
                Some(x) => x,
                None => return
            };
        }
        let mut new_ring = SnapRing::new(self.size);
        new_ring.push(item);
        self.rings.push(new_ring);
    }
    fn pop(&mut self) -> Option<Snapshot> {
        self
            .rings
            .iter_mut()
            .map(|ring| ring.pop())
            .filter_map(|snap| snap)
            .next()
    }
}

#[derive(Debug, Clone)]
struct SnapRing {
    succeed: bool,
    short_term: VecDeque<Snapshot>
}
impl SnapRing {
    fn new(size: usize) -> SnapRing {
        let short_term = VecDeque::with_capacity(size + 1);
        SnapRing {
            short_term,
            succeed: true
        }
    }
    fn push(&mut self, snap: Snapshot) -> Option<Snapshot> {
        self.short_term.push_back(snap);
        if self.short_term.len() >= self.short_term.capacity() {
            let succeed = self.succeed;
            self.succeed = !succeed;
            if succeed {
                return self.short_term.pop_front();
            }
        }
        None
    }
    fn pop(&mut self) -> Option<Snapshot> {
        self.short_term.pop_back()
    }
}
