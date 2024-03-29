use std::cmp::Ordering;
use std::collections::BinaryHeap;

use grid::{COST, Distance, Grid, Measure, Point, Tile};

#[derive(Debug)]
struct Node {
    point: Point,
    f: Distance,
    g: Distance,
}

impl Node {
    pub fn point(&self) -> &Point {
        &self.point
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.f.eq(&other.f) && self.g.eq(&other.g)
    }
}

impl Eq for Node {}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Node) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Node) -> Ordering {
        match other.f.partial_cmp(&self.f) {
            Some(Ordering::Equal) => {
                self.g.partial_cmp(&other.g).unwrap_or(Ordering::Equal)
            }
            Some(o) => o,
            None => Ordering::Equal,
        }
    }
}

pub type Path = Vec<Point>;

fn extract_path(grid: &Grid, end: Point) -> Path {
    let mut path = Path::new();

    let mut point = end;
    loop {
        let tile = &grid[&point];

        if let Some(previous) = tile.parent() {
            path.push(point);
            point = previous;
        } else {
            break;
        }
    }

    path
}

pub struct Data {
    pub path: Path,
    pub expansions: usize,
}

pub fn astar<H, P>(grid: &mut Grid,
                   source: &Point,
                   target: &Point,
                   heuristic: H,
                   passable: P)
                   -> Option<Data>
    where H: Fn(&Point, &Point) -> Distance,
          P: Fn(&Tile) -> bool
{
    let episode = grid.next_episode();

    let mut open = BinaryHeap::new();
    let mut expansions = 0;

    grid[source].visit_initial(Distance::octile_heuristic(source, target),
                               episode);
    open.push(Node {
                  point: *source,
                  f: grid[source].f(),
                  g: grid[source].g(),
              });

    while let Some(expand) = open.pop() {
        expansions += 1;
        let point = expand.point();
        if point == target {
            return Some(Data {
                            path: extract_path(grid, *point),
                            expansions: expansions,
                        });
        } else {
            let g = grid[point].g();
            for (i, neighbor) in point.neighbors().iter().enumerate() {
                if let Some(neighbor) = *neighbor {
                    if let Some(ref mut tile) = grid.get_mut(&neighbor) {
                        if !tile.visited(episode) && passable(tile) {
                            let h = heuristic(&neighbor, target);
                            tile.visit(*point, g + COST[i], h, episode);
                            open.push(Node {
                                          point: neighbor,
                                          f: tile.f(),
                                          g: tile.g(),
                                      });
                        }
                    }
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    use parser::grid_from_str;

    #[test]
    fn solve_with_astar() {
        let mut grid = grid_from_str("type octile
height 4
width 4
map
....
.TT.
.TT.
....");

        let path = astar(&mut grid,
                         &Point::new(0, 0),
                         &Point::new(0, 0),
                         Distance::octile_heuristic,
                         Tile::passable)
                .unwrap()
                .path;

        assert_eq!(path.len(), 0);

        let path = astar(&mut grid,
                         &Point::new(0, 0),
                         &Point::new(3, 3),
                         Distance::octile_heuristic,
                         Tile::passable)
                .unwrap()
                .path;

        assert_eq!(path.len(), 5);
    }
}
