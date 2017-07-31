use agent::Agent;
use grid::Grid;
use instance::{Data, Instance};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum Verbosity {
    Zero,
    One,
    Two,
}

impl Verbosity {
    pub fn new(value: u8) -> Verbosity {
        match value {
            0 => Verbosity::Zero,
            1 => Verbosity::One,
            2 => Verbosity::Two,
            _ => Verbosity::Two,
        }
    }
}

pub struct Experiment {
    grid: Grid,
    start: usize,
    end: usize,
    seed: usize,
    verbosity: Verbosity,
}

impl Experiment {
    pub fn new(grid: Grid,
               start: usize,
               end: usize,
               seed: usize,
               verbosity: Verbosity)
               -> Experiment {
        Experiment {
            grid: grid,
            start: start,
            end: end,
            seed: seed,
            verbosity: verbosity,
        }
    }

    pub fn run<A>(&mut self, agent: A) -> Vec<Option<Data>>
        where A: Agent
    {
        let mut instance = Instance::new(&mut self.grid, agent, self.verbosity);
        instance.run_trials(self.start, self.end, self.seed)
    }
}
