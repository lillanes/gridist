#![feature(test)]

extern crate grids;
extern crate test;

use grids::agent::{AlwaysAstar, RepeatedAstar};
use grids::experiment::{Experiment, Verbosity};
use grids::grid::{Distance, Measure};
use grids::parser::grid_from_file;

use test::Bencher;

#[bench]
fn repeated_astar_50_times(b: &mut Bencher) {
    let grid = grid_from_file("maps/Mini.map");

    let heuristic = Distance::octile;
    let cost = Distance::euclidean;

    let mut experiment = Experiment::new(grid,
                                         0,
                                         50,
                                         0,
                                         Verbosity::Zero);


    b.iter(|| {
        experiment.run(RepeatedAstar::new(heuristic, cost));
    })
}

#[bench]
fn always_astar_50_times(b: &mut Bencher) {
    let grid = grid_from_file("maps/Mini.map");

    let heuristic = Distance::octile;
    let cost = Distance::euclidean;

    let mut experiment = Experiment::new(grid,
                                         0,
                                         50,
                                         0,
                                         Verbosity::Zero);


    b.iter(|| {
        experiment.run(AlwaysAstar::new(heuristic, cost));
    })
}
