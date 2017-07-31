use grid::{Distance, Grid, Point, Tile};
use search::{astar, Path};

#[derive(Debug)]
pub struct Datum {
    pub action: Point,
    pub expansions: usize,
}

pub trait Agent {
    fn act(&mut self,
           grid: &mut Grid,
           location: &Point,
           target: &Point)
           -> Option<Datum>;

    fn cost(&self, source: &Point, target: &Point) -> Distance;

    fn reset(&mut self) {}
}

#[derive(Debug)]
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
           -> Option<Datum> {
        astar(grid,
              &location,
              &target,
              &self.heuristic,
              &self.cost,
              Tile::freespace)
                .and_then(|mut data| {
                    data.path.pop().map(|next| {
                                            Datum {
                                                action: next,
                                                expansions: data.expansions,
                                            }
                                        })
                })
    }

    fn cost(&self, source: &Point, target: &Point) -> Distance {
        (self.cost)(source, target)
    }
}

#[derive(Debug)]
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
                   target: &Point)
                   -> usize {
        astar(grid,
              location,
              target,
              &self.heuristic,
              &self.cost,
              Tile::freespace)
                .map_or(0, |data| {
                    self.path = Some(data.path);
                    data.expansions
                })
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
           -> Option<Datum> {
        if let Some(next) = self.follow_path() {
            if grid[&next].freespace() {
                return Some(Datum {
                                action: next,
                                expansions: 0,
                            });
            }
        }

        let expansions = self.update_path(grid, location, target);
        self.follow_path().map(|next| {
                                   Datum {
                                       action: next,
                                       expansions: expansions,
                                   }
                               })
    }

    fn cost(&self, source: &Point, target: &Point) -> Distance {
        (self.cost)(source, target)
    }

    fn reset(&mut self) {
        self.path = None;
    }
}
