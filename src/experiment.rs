use agent::Agent;
use grid::{Distance, Grid, Point, Tile};
use search::{astar, Path};

#[derive(Debug, Default)]
pub struct Data {
    pub cost: Distance,
    pub steps: usize,
    pub episodes: usize,
    pub expansions: usize,
}

#[derive(Debug)]
pub struct Experiment<'a, A> {
    grid: &'a mut Grid,
    agent: A,
    location: Point,
    data: Data,
}

impl<'a, A> Experiment<'a, A>
    where A: Agent
{
    pub fn new(grid: &'a mut Grid, agent: A) -> Experiment<'a, A> {
        Experiment {
            grid: grid,
            agent: agent,
            location: Point::new(0, 0),
            data: Data::default(),
        }
    }

    fn move_agent(&mut self, point: Point) {
        self.location = point;
        self.grid.look(&self.location);
        println!("moved to {}", point);
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

    pub fn run(&mut self, source: Point, target: Point) -> bool {
        self.agent.reset();
        self.location = source;
        self.grid.look(&self.location);

        while let Some(point) = self.agent.act(self.grid,
                                               &self.location,
                                               &target) {
            self.move_agent(point);
            if point == target {
                return true;
            }
        }
        return false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use agent::{AlwaysAstar, RepeatedAstar};
    use grid::Measure;
    use parser::grid_from_str;

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
        let mut experiment = Experiment::new(&mut grid, agent);

        assert!(experiment.run(start, goal));
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
        let mut experiment = Experiment::new(&mut grid, agent);

        assert!(experiment.run(start, goal));

    }
}
