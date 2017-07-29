use grid::{Distance, Grid, Point, Tile};
use search::{astar, Path};

pub trait Agent {
    fn act(&mut self,
           grid: &mut Grid,
           location: &Point,
           target: &Point)
           -> Option<Point>;

    fn reset(&mut self) {}
}

pub struct AlwaysAstar<H, C> {
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

impl<H, C> Agent for AlwaysAstar<H, C>
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

pub struct RepeatedAstar<H, C> {
    heuristic: H,
    cost: C,
    path: Option<Path>,
}

impl<H, C> RepeatedAstar<H, C>
    where H: Fn(&Point, &Point) -> Distance,
          C: Fn(&Point, &Point) -> Distance
{
    pub fn new(heuristic: H, cost: C) -> RepeatedAstar<H, C> {
        RepeatedAstar {
            heuristic: heuristic,
            cost: cost,
            path: None,
        }
    }

    fn update_path(&mut self,
                   grid: &mut Grid,
                   location: &Point,
                   target: &Point) {
        self.path = astar(grid,
                          location,
                          target,
                          &self.heuristic,
                          &self.cost,
                          Tile::freespace);
    }

    fn follow_path(&mut self) -> Option<Point> {
        self.path.as_mut().and_then(|path| path.pop())
    }
}

impl<H, C> Agent for RepeatedAstar<H, C>
    where H: Fn(&Point, &Point) -> Distance,
          C: Fn(&Point, &Point) -> Distance
{
    fn act(&mut self,
           grid: &mut Grid,
           location: &Point,
           target: &Point)
           -> Option<Point> {
        if let Some(next) = self.follow_path() {
            if grid[&next].freespace() {
                return Some(next);
            }
        }

        self.update_path(grid, location, target);
        self.follow_path()
    }

    fn reset(&mut self) {
        self.path = None;
    }
}
