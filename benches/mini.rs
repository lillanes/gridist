#[macro_use]
extern crate bencher;
extern crate gridist;

use bencher::Bencher;

use gridist::agent::{AlwaysAstar, RepeatedAstar};
use gridist::experiment::{Experiment, Verbosity};
use gridist::grid::{Distance, Measure};
use gridist::parser::grid_from_file;

fn mini_rastar(b: &mut Bencher) {
    let grid = grid_from_file("maps/Mini.map");

    let heuristic = Distance::octile;
    let cost = Distance::euclidean;

    let mut experiment = Experiment::trials(grid, 0, 50, 0, Verbosity::Zero);


    b.iter(|| { experiment.run(RepeatedAstar::new(heuristic, cost)) });
}

fn mini_astar(b: &mut Bencher) {
    let grid = grid_from_file("maps/Mini.map");

    let heuristic = Distance::octile;
    let cost = Distance::euclidean;

    let mut experiment = Experiment::trials(grid, 0, 50, 0, Verbosity::Zero);


    b.iter(|| { experiment.run(AlwaysAstar::new(heuristic, cost)) });
}

benchmark_group!(mini, mini_rastar, mini_astar);
benchmark_main!(mini);
