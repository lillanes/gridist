use grid::{Distance, Grid, Point, Tile};
use search::{astar, Path};

#[derive(Debug, Default)]
pub struct Data {
    pub cost: Distance,
    pub steps: usize,
    pub stages: usize,
    pub expansions: usize,
}

#[derive(Debug)]
pub struct Agent<'a, S> {
    data: Data,
    location: Point,
    grid: &'a mut Grid,
    strategy: S,
}

impl<'a, S> Agent<'a, S>
    where S: AgentStrategy
{
    pub fn new(grid: &'a mut Grid, strategy: S) -> Agent<'a, S> {
        Agent {
            data: Data::default(),
            location: Point::new(0, 0),
            grid: grid,
            strategy: strategy,
        }
    }

    fn move_to(&mut self, point: Point) {
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

    pub fn solve(&mut self, source: Point, target: Point) -> bool {
        self.location = source;
        self.grid.look(&self.location);

        while let Some(point) = self.strategy.act(self.grid,
                                                  &self.location,
                                                  &target) {
            self.move_to(point);
            if point == target {
                return true;
            }
        }
        return false;
    }
}

trait AgentStrategy {
    fn act(&mut self,
           grid: &mut Grid,
           location: &Point,
           target: &Point)
           -> Option<Point>;
}

struct AlwaysAstar<H, C> {
    heuristic: H,
    cost: C,
}

impl<H, C> AlwaysAstar<H, C> {
    pub fn new(heuristic: H, cost: C) -> AlwaysAstar<H, C> {
        AlwaysAstar {
            heuristic: heuristic,
            cost: cost,
        }
    }
}

impl<H, C> AgentStrategy for AlwaysAstar<H, C>
    where H: Fn(&Point, &Point) -> Distance,
          C: Fn(&Point, &Point) -> Distance
{
    fn act(&mut self,
           grid: &mut Grid,
           location: &Point,
           target: &Point)
           -> Option<Point> {
        astar(grid,
              &location,
              &target,
              &self.heuristic,
              &self.cost,
              Tile::freespace)
                .and_then(|mut path| path.pop())
    }
}

struct RepeatedAstar<H, C> {
    heuristic: H,
    cost: C,
    path: Path,
}

impl<H, C> RepeatedAstar<H, C>
    where H: Fn(&Point, &Point) -> Distance,
          C: Fn(&Point, &Point) -> Distance
{
    pub fn new(heuristic: H, cost: C) -> RepeatedAstar<H, C> {
        RepeatedAstar {
            heuristic: heuristic,
            cost: cost,
            path: Path::new(),
        }
    }

    fn update_path(&mut self, grid: &mut Grid, location: &Point, target: &Point) {
        self.path = astar(grid,
                          location,
                          target,
                          &self.heuristic,
                          &self.cost,
                          Tile::freespace).unwrap_or(Path::new());
    }
}

impl<H, C> AgentStrategy for RepeatedAstar<H, C>
    where H: Fn(&Point, &Point) -> Distance,
          C: Fn(&Point, &Point) -> Distance
{
    fn act(&mut self,
           grid: &mut Grid,
           location: &Point,
           target: &Point)
           -> Option<Point> {
        if let Some(next) = self.path.pop() {
            if grid[&next].freespace() {
                return Some(next);
            }
        }

        self.update_path(grid, location, target);
        self.path.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let strategy = AlwaysAstar::new(Distance::octile, Distance::euclidean);
        let mut agent = Agent::new(&mut grid, strategy);

        assert!(agent.solve(start, goal));
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

        let strategy = RepeatedAstar::new(Distance::octile,
                                          Distance::euclidean);
        let mut agent = Agent::new(&mut grid, strategy);

        assert!(agent.solve(start, goal));

    }

}
