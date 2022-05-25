use std::{iter::repeat_with, mem::replace};

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
    pub fn possible(&self) -> (Vec<Possibility>, Vec<u32>) {
        let base: Vec<_> = (0..self.starts.len()).map(|n| Possibility(n as u32)).collect();
        let mut possible_buf = Vec::new();
        let mut score_buf = Vec::new();
        self.check(&base, &[&base; 8], &mut possible_buf, &mut score_buf);
        (possible_buf, score_buf)
    }
    pub fn get_surrounds_and_weight(&self, possibility: Possibility) -> (&[[Possibility; 8]], &[u32]) {
        let index = possibility.0 as usize;
        let start = self.starts[index];
        let end = match self.starts.get(index + 1) {
            Some(n) => *n,
            None => self.starts.len()
        };
        (&self.surrounds[start..end], &self.weights[start..end])
    }
    pub fn check(&self, center: &[Possibility], surroundings: &[&[Possibility]; 8], possible_buf: &mut Vec<Possibility>, score_buf: &mut Vec<u32>) {
        possible_buf.clear();
        score_buf.clear();
        let p_s = center.iter().filter_map(|center_poss| {
            let (surrounds, weights) = self.get_surrounds_and_weight(center_poss.clone());
            let score: u32 = surrounds.iter()
                .zip(weights)
                .map(|(targets, weight)|  {
                    let matches = targets.iter().zip(surroundings).all(|(target, options)| {
                        options.contains(target)
                    });
                    match matches {
                        true => *weight,
                        false => 0
                    }
                })
                .sum();
            match score {
                0 => None,
                _ => Some((*center_poss, score))
            }
        });
        let mut duo = (replace(possible_buf, Vec::new()), replace(score_buf, Vec::new()));
        duo.extend(p_s);
        *possible_buf = duo.0;
        *score_buf = duo.1;
    }
}

#[test]
fn checking() {
    let ruleset = [
        Rule {
            tiles:
                [
                    0, 1, 0,
                    1, 0, 1,
                    0, 1, 0,
                ],
            weight: 1
        },
        Rule {
            tiles:
                [
                    1, 0, 1,
                    0, 1, 0,
                    1, 0, 1,
                ],
            weight: 1
        },
    ];
    let mut possible_buf = Vec::new();
    let mut score_buf = Vec::new();
    let rules = Rules::new(&ruleset);
    let (center, _) = rules.possible();
    rules.check(&center, &[&center; 8], &mut possible_buf, &mut score_buf);
    assert_eq!(possible_buf, center);
    let mut surroundings = [&center[..]; 8];
    surroundings[0] = &center[..1];
    rules.check(&center, &surroundings, &mut possible_buf, &mut score_buf);
    assert_eq!(possible_buf.len(), 1);
}
