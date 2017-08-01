use std::mem::replace;
use std::ops::Index;

use rand::{SeedableRng, StdRng};
use rand::distributions::{IndependentSample, Range};

use agent::Agent;
use experiment::Verbosity;
use grid::{Distance, Grid, Point};

#[derive(Debug, Default)]
pub struct Datum {
    pub cost: Distance,
    pub steps: usize,
    pub episodes: usize,
    pub expansions: usize,
}

#[derive(Debug, Default)]
pub struct Data(Vec<Option<Datum>>);

impl Data {
    pub fn new(capacity: usize) -> Data {
        Data(Vec::with_capacity(capacity))
    }

    pub fn push(&mut self, datum: Option<Datum>) {
        self.0.push(datum);
    }

    pub fn print(&self) {
        for (i, datum) in self.0.iter().enumerate() {
            print!("Trial {}: ", i);
            if let Some(ref datum) = *datum {
                println!("{} ({} steps, {} episodes, {} expansions)",
                         datum.cost,
                         datum.steps,
                         datum.episodes,
                         datum.expansions);
            } else {
                println!("<none>");
            }
        }
    }
}

impl Index<usize> for Data {
    type Output = Option<Datum>;

    fn index(&self, index: usize) -> &Option<Datum> {
        &self.0[index]
    }
}

#[derive(Debug)]
pub struct Instance<'a, A> {
    grid: &'a mut Grid,
    agent: A,
    location: Point,
    data: Datum,
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
            data: Datum::default(),
            verbosity: verbosity,
        }
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

    pub fn run_once(&mut self, source: Point, target: Point) -> Option<Datum> {
        self.data = Datum::default();
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
                return Some(replace(&mut self.data, Datum::default()));
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
                      -> Data {
        let trials = self.build_trials(start, end, seed);

        let mut results = Data::new(end - start);
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

        let results = instance.run_trials(98, 100, 0);

        let first = results[0].as_ref().unwrap();
        assert_eq!(first.steps, 4);
        assert_eq!(first.episodes, 2);

        let second = results[1].as_ref().unwrap();
        assert_eq!(second.steps, 3);
        assert_eq!(second.episodes, 1);

        let new_results = instance.run_trials(99, 100, 0);

        let new_result = new_results[0].as_ref().unwrap();

        assert_eq!(second.cost, new_result.cost);
        assert_eq!(second.steps, new_result.steps);
        assert_eq!(second.episodes, new_result.episodes);
        assert_eq!(second.expansions, new_result.expansions);
    }
}
