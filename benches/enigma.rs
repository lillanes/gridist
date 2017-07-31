#![feature(test)]

extern crate gridist;
extern crate test;

use gridist::agent::{AlwaysAstar, RepeatedAstar};
use gridist::experiment::{Experiment, Verbosity};
use gridist::grid::{Distance, Measure};
use gridist::parser::grid_from_file;

use test::Bencher;

#[bench]
fn repeated_astar_5_times(b: &mut Bencher) {
    let grid = grid_from_file("maps/Enigma.map");

    let heuristic = Distance::octile;
    let cost = Distance::euclidean;

    let mut experiment = Experiment::new(grid,
                                         0,
                                         5,
                                         0,
                                         Verbosity::Zero);


    b.iter(|| {
        experiment.run(RepeatedAstar::new(heuristic, cost));
    })
}

#[bench]
fn always_astar_5_times(b: &mut Bencher) {
    let grid = grid_from_file("maps/Enigma.map");

    let heuristic = Distance::octile;
    let cost = Distance::euclidean;

    let mut experiment = Experiment::new(grid,
                                         0,
                                         5,
                                         0,
                                         Verbosity::Zero);


    b.iter(|| {
        experiment.run(AlwaysAstar::new(heuristic, cost));
    })
}
