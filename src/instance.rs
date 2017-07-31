use agent::Agent;
use grid::{Distance, Grid, Point};

use std::mem::replace;

use rand::{SeedableRng, StdRng};
use rand::distributions::{IndependentSample, Range};

#[derive(Debug, Default, PartialEq)]
pub struct Data {
    pub cost: Distance,
    pub steps: usize,
    pub episodes: usize,
    pub expansions: usize,
}

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

#[derive(Debug)]
pub struct Instance<'a, A> {
    grid: &'a mut Grid,
    agent: A,
    location: Point,
    data: Data,
    verbosity: Verbosity,
}

impl<'a, A> Instance<'a, A>
    where A: Agent
{
    pub fn new(grid: &'a mut Grid,
               agent: A,
               verbosity: Verbosity)
               -> Instance<'a, A> {
        Instance {
            grid: grid,
            agent: agent,
            location: Point::new(0, 0),
            data: Data::default(),
            verbosity: verbosity,
        }
    }

    pub fn set_verbosity(&mut self, verbosity: Verbosity) {
        self.verbosity = verbosity;
    }

    fn move_agent(&mut self, point: Point) {
        self.data.steps += 1;
        self.data.cost += self.agent.cost(&self.location, &point);
        self.location = point;
        self.grid.look(&self.location);
    }

    fn print(&self, target: &Point) {
        for (y, row) in self.grid.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                let point = Point::new(y, x);
                if self.location == point {
                    print!("a");
                } else if *target == point {
                    print!("*");
                } else {
                    print!("{}", cell.belief());
                }
            }
            println!();
        }
        println!();
    }

    pub fn run_once(&mut self, source: Point, target: Point) -> Option<Data> {
        self.data = Data::default();
        self.agent.reset();
        self.location = source;
        self.grid.look(&self.location);

        while let Some(datum) = self.agent.act(self.grid,
                                               &self.location,
                                               &target) {
            if self.verbosity >= Verbosity::Two {
                self.print(&target);
            }

            if datum.expansions > 0 {
                self.data.episodes += 1;
                self.data.expansions += datum.expansions;
            }

            self.move_agent(datum.action);

            if datum.action == target {
                if self.verbosity >= Verbosity::Two {
                    self.print(&target);
                }
                return Some(replace(&mut self.data, Data::default()));
            }
        }
        return None;
    }

    fn build_trials(&mut self,
                    start: usize,
                    end: usize,
                    seed: usize)
                    -> Vec<(Point, Point)> {
        let mut rng: StdRng = SeedableRng::from_seed([seed,
                                                      self.grid.height(),
                                                      self.grid.width()]
                                                             .as_ref());
        let mut trials = Vec::with_capacity(end - start);
        let yrange = Range::new(0, self.grid.height());
        let xrange = Range::new(0, self.grid.width());

        for trial_idx in 0..end {
            loop {
                let source = Point::new(yrange.ind_sample(&mut rng),
                                        xrange.ind_sample(&mut rng));
                let target = Point::new(yrange.ind_sample(&mut rng),
                                        xrange.ind_sample(&mut rng));
                if source != target && self.grid[&source].passable() &&
                   self.grid[&target].passable() &&
                   self.grid.has_path(&source, &target) {
                    if trial_idx >= start {
                        trials.push((source, target));
                    }
                    break;
                }
            }
        }
        trials
    }

    pub fn run_trials(&mut self,
                      start: usize,
                      end: usize,
                      seed: usize)
                      -> Vec<Option<Data>> {
        let trials = self.build_trials(start, end, seed);

        let mut results = Vec::with_capacity(end - start);
        for trial in trials.iter() {
            if self.verbosity >= Verbosity::One {
                println!("Running search from {} to {}.", trial.0, trial.1);
            }
            self.grid.forget();
            results.push(self.run_once(trial.0, trial.1));
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use agent::{AlwaysAstar, RepeatedAstar};
    use grid::Measure;
    use parser::grid_from_str;

    use std::f64::consts::SQRT_2;

    #[test]
    fn always_astar() {
        let mut grid = grid_from_str("type octile
height 4
width 4
map
....
.TT.
.TT.
....");

        let start = Point::new(0, 0);
        let goal = Point::new(3, 3);

        let agent = AlwaysAstar::new(Distance::octile, Distance::euclidean);
        let mut instance = Instance::new(&mut grid, agent, Verbosity::Two);

        let results = instance.run_once(start, goal).unwrap();

        assert_eq!(results.steps, 5);
        assert_eq!(results.cost, 4.0 + SQRT_2);
        assert_eq!(results.episodes, 5);
    }

    #[test]
    fn repeated_astar() {
        let mut grid = grid_from_str("type octile
height 4
width 4
map
....
.TT.
.TT.
....");

        let start = Point::new(0, 0);
        let goal = Point::new(3, 3);

        let agent = RepeatedAstar::new(Distance::octile, Distance::euclidean);
        let mut instance = Instance::new(&mut grid, agent, Verbosity::Two);

        let results = instance.run_once(start, goal).unwrap();

        assert_eq!(results.steps, 5);
        assert_eq!(results.cost, 4.0 + SQRT_2);
        assert_eq!(results.episodes, 2);
    }

    #[test]
    fn repeated_astar_trials() {
        let mut grid = grid_from_str("type octile
height 4
width 4
map
....
.TT.
.TT.
....");

        let agent = RepeatedAstar::new(Distance::octile, Distance::euclidean);
        let mut instance = Instance::new(&mut grid, agent, Verbosity::Two);
        instance.set_verbosity(Verbosity::Two);

        let results = instance.run_trials(98, 100, 0);
        let results =
            results.into_iter().map(|e| e.unwrap()).collect::<Vec<_>>();

        assert_eq!(results[0].steps, 4);
        assert_eq!(results[0].episodes, 2);

        assert_eq!(results[1].steps, 3);
        assert_eq!(results[1].episodes, 1);

        let new_results = instance.run_trials(99, 100, 0);
        let new_results =
            new_results.into_iter().map(|e| e.unwrap()).collect::<Vec<_>>();

        assert_eq!(results[1], new_results[0]);
    }
}
