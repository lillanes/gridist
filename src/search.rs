use std::cmp::Ordering;
use std::collections::BinaryHeap;

use grid::{Distance, Grid, Measure, Point, Tile};

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
        self.f.eq(&other.f)
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
        let cell = &grid[&point];

        if let Some(previous) = cell.parent() {
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

pub fn astar<H, D, P>(grid: &mut Grid,
                      source: &Point,
                      target: &Point,
                      heuristic: H,
                      cost: D,
                      passable: P)
                      -> Option<Data>
    where H: Fn(&Point, &Point) -> Distance,
          D: Fn(&Point, &Point) -> Distance,
          P: Fn(&Tile) -> bool
{
    grid.restage();

    let mut open = BinaryHeap::new();
    let mut expansions = 0;

    grid[source].visit_initial(Distance::octile(source, target));
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
            for neighbor in point.neighbors().iter() {
                if let Some(ref mut tile) = grid.get_mut(neighbor) {
                    if !tile.visited() && passable(tile) {
                        let h = heuristic(neighbor, target);
                        tile.visit(*point, g + cost(point, neighbor), h);
                        open.push(Node {
                                      point: *neighbor,
                                      f: tile.f(),
                                      g: tile.g(),
                                  });
                    }
                }
            }
        }
    }
    return None;
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
                         Distance::octile,
                         Distance::euclidean,
                         Tile::passable)
                .unwrap()
                .path;

        assert_eq!(path.len(), 0);

        let path = astar(&mut grid,
                         &Point::new(0, 0),
                         &Point::new(3, 3),
                         Distance::octile,
                         Distance::euclidean,
                         Tile::passable)
                .unwrap()
                .path;

        assert_eq!(path.len(), 5);
    }
}
