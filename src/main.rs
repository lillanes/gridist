extern crate gridist;

use gridist::arguments::run_experiment_from_cli;

fn main() {
    let data = run_experiment_from_cli();

    data.print();
}
