#[macro_use]
extern crate bencher;
extern crate gridist;

use bencher::Bencher;

use gridist::agent::{AlwaysAstar, RepeatedAstar};
use gridist::experiment::{Experiment, Verbosity};
use gridist::grid::{Distance, Measure};
use gridist::parser::grid_from_file;

fn run_enigma_rastar(b: &mut Bencher) {
    let grid = grid_from_file("maps/Enigma.map");

    let heuristic = Distance::octile_heuristic;

    let mut experiment = Experiment::trials(grid, 0, 5, 0, Verbosity::Zero);

    b.iter(|| { experiment.run(RepeatedAstar::new(heuristic)) });
}

fn enigma_rastar(b: &mut Bencher) {
    b.bench_n(1, |b| { run_enigma_rastar(b); });
}

fn run_enigma_astar(b: &mut Bencher) {
    let grid = grid_from_file("maps/Enigma.map");

    let heuristic = Distance::octile_heuristic;

    let mut experiment = Experiment::trials(grid, 0, 5, 0, Verbosity::Zero);

    b.iter(|| { experiment.run(AlwaysAstar::new(heuristic)) });
}

fn enigma_astar(b: &mut Bencher) {
    b.bench_n(1, |b| { run_enigma_astar(b); });
}

benchmark_group!(enigma, enigma_astar, enigma_rastar);
benchmark_main!(enigma);
