use agent::Agent;
use grid::{Distance, Grid, Point, Tile};
use search::{astar, Path};

use std::mem::replace;

#[derive(Debug, Default)]
pub struct Data {
    pub cost: Distance,
    pub steps: usize,
    pub episodes: usize,
    pub expansions: usize,
}

#[derive(Debug)]
pub struct Experiment<'a, A, C> {
    grid: &'a mut Grid,
    cost: C,
    agent: A,
    location: Point,
    data: Data,
}

impl<'a, A, C> Experiment<'a, A, C>
    where A: Agent,
          C: Fn(&Point, &Point) -> Distance
{
    pub fn new(grid: &'a mut Grid, agent: A, cost: C) -> Experiment<'a, A, C> {
        Experiment {
            grid: grid,
            cost: cost,
            agent: agent,
            location: Point::new(0, 0),
            data: Data::default(),
        }
    }

    fn move_agent(&mut self, point: Point) {
        self.data.steps += 1;
        self.data.cost += (self.cost)(&self.location, &point);
        self.location = point;
        self.grid.look(&self.location);
    }

    fn print(&self) {
        for (y, row) in self.grid.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if self.location == Point::new(y, x) {
                    print!("a");
                } else {
                    print!("{}", cell);
                }
            }
            println!();
        }
    }

    pub fn run(&mut self, source: Point, target: Point) -> Option<Data> {
        self.data = Data::default();
        self.agent.reset();
        self.location = source;
        self.grid.look(&self.location);

        while let Some(datum) = self.agent.act(self.grid,
                                               &self.location,
                                               &target) {
            if datum.expansions > 0 {
                self.data.episodes += 1;
                self.data.expansions += datum.expansions;
            }

            self.move_agent(datum.action);

            if datum.action == target {
                return Some(replace(&mut self.data, Data::default()));
            }
        }
        return None;
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
        let mut experiment =
            Experiment::new(&mut grid, agent, Distance::euclidean);

        let results = experiment.run(start, goal).unwrap();

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
        let mut experiment =
            Experiment::new(&mut grid, agent, Distance::euclidean);

        let results = experiment.run(start, goal).unwrap();

        assert_eq!(results.steps, 5);
        assert_eq!(results.cost, 4.0 + SQRT_2);
        assert_eq!(results.episodes, 2);
    }
}
