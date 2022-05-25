use std::iter::repeat_with;

use crate::tiles::Possibility;

#[derive(Debug, Clone)]
pub struct Rule {
    pub tiles: [u32; 9],
    pub weight: u32
}

#[derive(Debug, Clone)]
pub struct Rules {
    surrounds: Vec<[Possibility; 8]>,
    weights: Vec<u32>,
    starts: Vec<usize>
}
impl Rules {
    pub fn new(rules: &[Rule]) -> Rules {
        let mut proto_surrounds: Vec<Vec<_>> = Vec::new();
        for rule in rules {
            let mut surround = [0; 8];
            surround[..4].clone_from_slice(&rule.tiles[..4]);
            surround[4..].clone_from_slice(&rule.tiles[5..]);

            let center = rule.tiles[4] as usize;
            
            match proto_surrounds.get_mut(center) {
                Some(v) => v.push((surround, rule.weight)),
                None => {
                    let diff = center - proto_surrounds.len();
                    proto_surrounds.extend(repeat_with(Vec::new).take(diff));
                    proto_surrounds.push(vec![(surround, rule.weight)]);
                }
            }
        }

        let mut surrounds = Vec::with_capacity(proto_surrounds.iter().map(Vec::len).sum());
        let mut weights = Vec::with_capacity(surrounds.capacity());
        let mut starts = Vec::with_capacity(proto_surrounds.len());
        for ps in proto_surrounds.iter() {
            let start = surrounds.len();
            starts.push(start);
            let iter = ps.iter().map(|(s, _w)| s.map(|n| Possibility(n)));
            surrounds.extend(iter);
            let weights_iter = ps.iter().map(|(_s, w)| *w);
            weights.extend(weights_iter);
        }
        Rules { surrounds, weights, starts }
    }
    pub(crate) fn num_starts(&self) -> usize {
        self.starts.len()
    }
    pub fn get_surrounds_and_weight(&self, possibility: Possibility) -> (&[[Possibility; 8]], &[u32]) {
        let index = possibility.0 as usize;
        let start = self.starts[index];
        let end = match self.starts.get(index) {
            Some(n) => *n,
            None => self.starts.len()
        };
        (&self.surrounds[start..end], &self.weights[start..end])
    }
    pub fn check(&self, center: &[Possibility], surroundings: &[&[Possibility]; 8], possible_buf: &mut Vec<Possibility>, score_buf: &mut Vec<u32>) {
        possible_buf.clear();
        score_buf.clear();
        let scored_retained = center.iter().map(|&possibility| {
            let (surrounds, weights) = self.get_surrounds_and_weight(possibility);
            let score = surrounds
                .iter()
                .zip(weights)
                .filter(|&(target, _)| {
                    target
                        .iter()
                        .zip(surroundings)
                        .any(|(cell_target, options)| {
                            options.contains(cell_target)
                        })
                })
                .map(|(_, n)| *n)
                .sum::<u32>();
            (possibility, score)
        }).filter(|(_, score)| *score != 0);
        for (retained, score) in scored_retained {
            possible_buf.push(retained);
            score_buf.push(score);
        }
    }
}
