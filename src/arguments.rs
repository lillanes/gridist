use docopt::Docopt;
use serde::de;

use agent::{AlwaysAstar, RepeatedAstar};
use experiment::{Experiment, Verbosity};
use instance::Data;
use grid::{Distance, Measure, Point};
use parser::grid_from_file;

const USAGE: &'static str = "
Usage:
    gridist <map> <trials> [--algorithm=<algorithm>] [--heuristic=<heuristic>] [--cost=<cost>] [--verbosity=<verbosity>] [--from=<from>] [--seed=<seed>]
    gridist --help

Options:
    -h, --help               Show this screen.
    --algorithm=<algorithm>  The algorithm to use [default: rastar].
    --heuristic=<heuristic>  The heuristic function to use [default: octile].
    --cost=<distance>        The cost metric to use [default: euclidean].
    --verbosity=<verbosity>  Level of verbosity [0-2] [default: 1].
    --from=<from>            Trial index at which to start running [default: 0].
    --seed=<seed>            A seed for generating random trials.

Algorithms:
    astar        Do a full A* search at every step.
    rastar       Do a full A* search and follow as long as possible.

Heuristics and cost metrics:
    euclidean  The Euclidean distance metric (sqrt(dy^2+dx^2)).
    octile     The octile distance metric (max(dy,dx)-min(dy,dx)+sqrt(2)*min(dy,dx)).
";

#[derive(Debug, Deserialize)]
enum Algorithm {
    Astar,
    Rastar,
}

#[derive(Debug, Deserialize)]
enum DistanceMetric {
    Euclidean,
    Octile,
}

impl<'de> de::Deserialize<'de> for Verbosity {
    fn deserialize<D>(deserializer: D) -> Result<Verbosity, D::Error>
        where D: de::Deserializer<'de>
    {
        Ok(Verbosity::new(u8::deserialize(deserializer)?))
    }
}

#[derive(Debug, Deserialize)]
struct Args {
    arg_map: String,
    arg_trials: usize,
    flag_algorithm: Algorithm,
    flag_heuristic: DistanceMetric,
    flag_cost: DistanceMetric,
    flag_verbosity: Verbosity,
    flag_from: usize,
    flag_seed: usize,
}

fn get_distance(argument: DistanceMetric) -> (fn(&Point, &Point) -> Distance) {
    match argument {
        DistanceMetric::Euclidean => Distance::euclidean,
        DistanceMetric::Octile => Distance::octile,
    }
}

fn run_experiment_from_args(args: Args) -> Data {
    let grid = grid_from_file(&args.arg_map);
    let heuristic = get_distance(args.flag_heuristic);
    let cost = get_distance(args.flag_cost);

    let mut experiment = Experiment::new(grid,
                                         args.flag_from,
                                         args.flag_from + args.arg_trials,
                                         args.flag_seed,
                                         args.flag_verbosity);

    match args.flag_algorithm {
        Algorithm::Astar => experiment.run(AlwaysAstar::new(heuristic, cost)),
        Algorithm::Rastar => {
            experiment.run(RepeatedAstar::new(heuristic, cost))
        }
    }
}

pub fn run_experiment_from_cli() -> Data {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    run_experiment_from_args(args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_experiment() {
        let argv = vec!["gridist", "maps/Mini.map", "2", "--seed=10"];
        let args: Args = Docopt::new(USAGE)
            .and_then(|d| d.argv(argv.into_iter()).deserialize())
            .unwrap();

        run_experiment_from_args(args);
    }
}
