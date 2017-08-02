use agent::Agent;
use grid::{Grid, Point};
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
            _ => Verbosity::Two,
        }
    }
}

struct RandomTrialData {
    start: usize,
    end: usize,
    seed: usize,
}

struct PointPair {
    source: Point,
    target: Point,
}

enum Configuration {
    Trials(RandomTrialData),
    Single(PointPair),
}

pub struct Experiment {
    grid: Grid,
    config: Configuration,
    verbosity: Verbosity,
}

impl Experiment {
    pub fn trials(grid: Grid,
                  start: usize,
                  end: usize,
                  seed: usize,
                  verbosity: Verbosity)
                  -> Experiment {
        Experiment {
            grid: grid,
            config: Configuration::Trials(RandomTrialData {
                                              start: start,
                                              end: end,
                                              seed: seed,
                                          }),
            verbosity: verbosity,
        }
    }

    pub fn single(grid: Grid,
                  source: Point,
                  target: Point,
                  verbosity: Verbosity)
                  -> Experiment {
        Experiment {
            grid: grid,
            config: Configuration::Single(PointPair {
                                              source: source,
                                              target: target,
                                          }),
            verbosity: verbosity,
        }
    }

    pub fn run<A>(&mut self, agent: A) -> Data
        where A: Agent
    {
        let mut instance = Instance::new(&mut self.grid, agent, self.verbosity);

        match self.config {
            Configuration::Trials(ref trials) => {
                instance.run_trials(trials.start, trials.end, trials.seed)
            }
            Configuration::Single(ref single) => {
                let mut data = Data::new(1);
                data.push(instance.run_once(single.source, single.target));
                data
            }
        }
    }
}
