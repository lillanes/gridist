use std::cmp::{max, min};
use std::f64::consts::SQRT_2;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::ops::{Index, IndexMut};
use std::slice::Iter;

use search::astar;

pub const COST: [Distance; 8] = [SQRT_2, 1.0, SQRT_2, 1.0, 1.0, SQRT_2, 1.0,
                                 SQRT_2];

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Point {
    pub y: usize,
    pub x: usize,
}

impl Point {
    pub fn new(y: usize, x: usize) -> Point {
        Point { y: y, x: x }
    }

    pub fn x(&self) -> usize {
        self.x
    }

    pub fn y(&self) -> usize {
        self.y
    }

    pub fn neighbors(&self) -> [Option<Point>; 8] {
        let mut ns = [None; 8];
        if self.y > 0 {
            if self.x > 0 {
                ns[0] = Some(Point::new(self.y - 1, self.x - 1));
            }
            ns[1] = Some(Point::new(self.y - 1, self.x));
            ns[2] = Some(Point::new(self.y - 1, self.x + 1));
        }
        if self.x > 0 {
            ns[3] = Some(Point::new(self.y, self.x - 1));
            ns[5] = Some(Point::new(self.y + 1, self.x - 1));
        }
        ns[4] = Some(Point::new(self.y, self.x + 1));
        ns[6] = Some(Point::new(self.y + 1, self.x));
        ns[7] = Some(Point::new(self.y + 1, self.x + 1));

        ns
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "({}, {})", self.y, self.x)
    }
}

pub trait Measure {
    fn euclidean_heuristic(from: &Point, to: &Point) -> Self;

    fn octile_heuristic(from: &Point, to: &Point) -> Self;
}

pub type Distance = f64;

impl Measure for Distance {
    fn euclidean_heuristic(from: &Point, to: &Point) -> Distance {
        let dy = to.y as Distance - from.y as Distance;
        let dx = to.x as Distance - from.x as Distance;

        (dy * dy + dx * dx).sqrt()
    }

    fn octile_heuristic(from: &Point, to: &Point) -> Distance {
        let dy = if to.y > from.y {
            to.y - from.y
        } else {
            from.y - to.y
        };
        let dx = if to.x > from.x {
            to.x - from.x
        } else {
            from.x - to.x
        };

        let cartesian = max(dy, dx) as Distance;
        let diagonal = min(dy, dx) as Distance;

        cartesian - diagonal + SQRT_2 * diagonal
    }
}

#[derive(Debug, PartialEq)]
pub enum Terrain {
    Ground,
    OutOfBounds,
    Trees,
    Swamp,
    Water,
}

impl Terrain {
    fn passable(&self) -> bool {
        *self == Terrain::Ground
    }
}

impl Display for Terrain {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            Terrain::Ground => write!(f, "."),
            Terrain::OutOfBounds => write!(f, "@"),
            Terrain::Trees => write!(f, "T"),
            Terrain::Swamp => write!(f, "S"),
            Terrain::Water => write!(f, "W"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Belief {
    Unknown,
    Passable,
    Impassable,
}

impl Display for Belief {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            Belief::Unknown => write!(f, "?"),
            Belief::Passable => write!(f, "."),
            Belief::Impassable => write!(f, "X"),
        }
    }
}

#[derive(Debug)]
pub struct Tile {
    terrain: Terrain,
    belief: Belief,
    parent: Option<Point>,
    g: Distance,
    h: Distance,
    visited: usize,
}

impl Tile {
    pub fn new(terrain: Terrain) -> Tile {
        Tile {
            terrain: terrain,
            belief: Belief::Unknown,
            parent: None,
            g: 0.0,
            h: 0.0,
            visited: 0,
        }
    }

    pub fn look(&mut self) {
        if self.belief == Belief::Unknown {
            if self.terrain.passable() {
                self.belief = Belief::Passable;
            } else {
                self.belief = Belief::Impassable;
            }
        }
    }

    pub fn passable(&self) -> bool {
        self.terrain.passable()
    }

    pub fn belief(&self) -> &Belief {
        &self.belief
    }

    pub fn freespace(&self) -> bool {
        self.belief != Belief::Impassable
    }

    pub fn parent(&self) -> Option<Point> {
        self.parent
    }

    pub fn f(&self) -> Distance {
        self.h + self.g
    }

    pub fn g(&self) -> Distance {
        self.g
    }

    pub fn visited(&self, episode: usize) -> bool {
        self.visited == episode
    }

    pub fn visit(&mut self,
                 parent: Point,
                 g: Distance,
                 h: Distance,
                 episode: usize) {
        self.parent = Some(parent);
        self.visited = episode;
        self.g = g;
        self.h = h;
    }

    pub fn visit_initial(&mut self, h: Distance, episode: usize) {
        self.parent = None;
        self.visited = episode;
        self.g = 0.0;
        self.h = h;
    }

    pub fn forget(&mut self) {
        self.belief = Belief::Unknown;
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.terrain)
    }
}

#[derive(Debug)]
pub struct Grid {
    tiles: Vec<Vec<Tile>>,
    episode: usize,
}

impl Grid {
    pub fn new(tiles: Vec<Vec<Tile>>) -> Grid {
        Grid {
            tiles: tiles,
            episode: 0,
        }
    }

    pub fn get(&self, point: &Point) -> Option<&Tile> {
        self.tiles.get(point.y()).and_then(|row| row.get(point.x()))
    }

    pub fn get_mut(&mut self, point: &Point) -> Option<&mut Tile> {
        self.tiles.get_mut(point.y()).and_then(|row| row.get_mut(point.x()))
    }

    pub fn next_episode(&mut self) -> usize {
        self.episode += 1;
        self.episode
    }

    pub fn forget(&mut self) {
        for row in &mut self.tiles {
            for cell in row.iter_mut() {
                cell.forget();
            }
        }
    }

    pub fn look(&mut self, point: &Point) {
        self.get_mut(point).map(|p| p.look());
        for neighbor in &point.neighbors() {
            if let Some(ref mut tile) =
                neighbor.and_then(|n| self.get_mut(&n)) {
                tile.look();
            }
        }
    }

    pub fn iter(&self) -> Iter<Vec<Tile>> {
        self.tiles.iter()
    }

    pub fn height(&self) -> usize {
        self.tiles.len()
    }

    pub fn width(&self) -> usize {
        self.tiles.get(0).map_or(0, |row| row.len())
    }

    pub fn has_path(&mut self, source: &Point, target: &Point) -> bool {
        astar(self,
              source,
              target,
              Distance::octile_heuristic,
              Tile::passable)
                .is_some()
    }
}

impl<'a> Index<&'a Point> for Grid {
    type Output = Tile;

    fn index(&self, index: &'a Point) -> &Tile {
        self.get(index).unwrap()
    }
}

impl<'a> IndexMut<&'a Point> for Grid {
    fn index_mut(&mut self, index: &'a Point) -> &mut Tile {
        self.get_mut(index).unwrap()
    }
}

impl Display for Grid {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        for row in &self.tiles {
            for tile in row.iter() {
                write!(f, "{}", tile)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn octile_heuristic_distance() {
        let p0 = Point::new(0, 0);
        let p1 = Point::new(0, 1);
        let p2 = Point::new(1, 1);
        let p3 = Point::new(5, 5);

        assert_eq!(Distance::octile_heuristic(&p0, &p1), 1.0);
        assert_eq!(Distance::octile_heuristic(&p0, &p2), SQRT_2);
        assert_eq!(Distance::octile_heuristic(&p1, &p3), 1.0 + 4.0 * SQRT_2);
    }

    #[test]
    fn euclidean_heuristic_distance() {
        let p0 = Point::new(0, 0);
        let p1 = Point::new(0, 1);
        let p2 = Point::new(1, 1);
        let p3 = Point::new(5, 5);

        assert_eq!(Distance::euclidean_heuristic(&p0, &p1), 1.0);
        assert_eq!(Distance::euclidean_heuristic(&p0, &p2), SQRT_2);
        assert_eq!(Distance::euclidean_heuristic(&p2, &p3), 4.0 * SQRT_2);
    }

    #[test]
    fn neighbors() {
        let corner = Point::new(0, 0);
        let corner_neighbors = [None,
                                None,
                                None,
                                None,
                                Some(Point::new(0, 1)),
                                None,
                                Some(Point::new(1, 0)),
                                Some(Point::new(1, 1))];
        assert_eq!(corner.neighbors(), corner_neighbors);

        let top = Point::new(0, 10);
        let top_neighbors = [None,
                             None,
                             None,
                             Some(Point::new(0, 9)),
                             Some(Point::new(0, 11)),
                             Some(Point::new(1, 9)),
                             Some(Point::new(1, 10)),
                             Some(Point::new(1, 11))];
        assert_eq!(top.neighbors(), top_neighbors);

        let left = Point::new(10, 0);
        let left_neighbors = [None,
                              Some(Point::new(9, 0)),
                              Some(Point::new(9, 1)),
                              None,
                              Some(Point::new(10, 1)),
                              None,
                              Some(Point::new(11, 0)),
                              Some(Point::new(11, 1))];
        assert_eq!(left.neighbors(), left_neighbors);

        let inner = Point::new(10, 10);
        let inner_neighbors = [Some(Point::new(9, 9)),
                               Some(Point::new(9, 10)),
                               Some(Point::new(9, 11)),
                               Some(Point::new(10, 9)),
                               Some(Point::new(10, 11)),
                               Some(Point::new(11, 9)),
                               Some(Point::new(11, 10)),
                               Some(Point::new(11, 11))];
        assert_eq!(inner.neighbors(), inner_neighbors);
    }
}
