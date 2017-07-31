extern crate docopt;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;

mod agent;
mod arguments;
mod experiment;
mod grid;
mod instance;
mod parser;
mod search;

use arguments::run_experiment_from_cli;

fn main() {
    run_experiment_from_cli();
}
