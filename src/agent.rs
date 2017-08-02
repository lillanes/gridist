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

    fn reset(&mut self) {}
}

#[derive(Debug)]
pub struct AlwaysAstar<H> {
    heuristic: H,
}

impl<H> AlwaysAstar<H> {
    pub fn new(heuristic: H) -> AlwaysAstar<H> {
        AlwaysAstar { heuristic: heuristic }
    }
}

impl<H> Agent for AlwaysAstar<H>
    where H: Fn(&Point, &Point) -> Distance
{
    fn act(&mut self,
           grid: &mut Grid,
           location: &Point,
           target: &Point)
           -> Option<Datum> {
        astar(grid, &location, &target, &self.heuristic, Tile::freespace)
            .and_then(|mut data| {
                data.path.pop().map(|next| {
                    Datum {
                        action: next,
                        expansions: data.expansions,
                    }
                })
            })
    }
}

#[derive(Debug)]
pub struct RepeatedAstar<H> {
    heuristic: H,
    path: Option<Path>,
}

impl<H> RepeatedAstar<H>
    where H: Fn(&Point, &Point) -> Distance
{
    pub fn new(heuristic: H) -> RepeatedAstar<H> {
        RepeatedAstar {
            heuristic: heuristic,
            path: None,
        }
    }

    fn update_path(&mut self,
                   grid: &mut Grid,
                   location: &Point,
                   target: &Point)
                   -> usize {
        astar(grid, location, target, &self.heuristic, Tile::freespace)
            .map_or(0, |data| {
                self.path = Some(data.path);
                data.expansions
            })
    }

    fn follow_path(&mut self) -> Option<Point> {
        self.path.as_mut().and_then(|path| path.pop())
    }
}

impl<H> Agent for RepeatedAstar<H>
    where H: Fn(&Point, &Point) -> Distance
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

    fn reset(&mut self) {
        self.path = None;
    }
}
