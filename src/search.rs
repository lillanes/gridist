use grid::{Distance, Grid, Measure, Point};

use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Debug)]
struct Node {
    point: Point,
    f: Distance,
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
            Some(o) => o,
            None => Ordering::Equal,
        }
    }
}

pub fn astar<H, D>(grid: &mut Grid,
                   source: Point,
                   target: Point,
                   heuristic: H,
                   cost: D)
                   -> bool
    where H: Fn(&Point, &Point) -> Distance,
          D: Fn(&Point, &Point) -> Distance
{
    grid.reset();

    let mut open = BinaryHeap::new();

    grid[&source].visit(0.0, Distance::octile(&source, &target));
    open.push(Node {
                  point: source,
                  f: grid[&source].f(),
              });

    while let Some(expand) = open.pop() {
        let point = expand.point();
        if *point == target {
            return true;
        } else {
            for neighbor in point.neighbors().iter() {
                let tile = &mut grid[neighbor];
                if !tile.visited() {
                    let g = tile.g() + cost(point, neighbor);
                    let h = heuristic(neighbor, &target);
                    tile.visit(g, h);
                    open.push(Node {
                                  point: *neighbor,
                                  f: g + h,
                              });
                }
            }
        }
    }
    return false;
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

        assert!(astar(&mut grid,
                      Point::new(0, 0),
                      Point::new(0, 0),
                      Distance::octile,
                      Distance::euclidean));
        assert!(astar(&mut grid,
                      Point::new(0, 0),
                      Point::new(3, 3),
                      Distance::octile,
                      Distance::euclidean));
    }
}
