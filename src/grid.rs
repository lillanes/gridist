use std::cmp::{max, min};
use std::f64::consts::SQRT_2;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::ops::{Index, IndexMut};
use std::slice::Iter;

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

    pub fn neighbors(&self) -> Vec<Point> {
        let yrange = if self.y < 1 { 0 } else { self.y - 1 }..self.y + 2;
        let xrange = if self.x < 1 { 0 } else { self.x - 1 }..self.x + 2;
        let mut neighbors = Vec::with_capacity(8);
        for y in yrange {
            for x in xrange.clone() {
                if y == self.y && x == self.x {
                    continue;
                }
                neighbors.push(Point::new(y, x));
            }
        }
        neighbors
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "({}, {})", self.y, self.x)
    }
}

pub trait Measure {
    fn euclidean(from: &Point, to: &Point) -> Self;

    fn octile(from: &Point, to: &Point) -> Self;
}

pub type Distance = f64;

impl Measure for Distance {
    fn euclidean(from: &Point, to: &Point) -> Distance {
        let dy = to.y as Distance - from.y as Distance;
        let dx = to.x as Distance - from.x as Distance;

        (dy * dy + dx * dx).sqrt()
    }

    fn octile(from: &Point, to: &Point) -> Distance {
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
    visited: bool,
}

impl Tile {
    pub fn new(terrain: Terrain) -> Tile {
        Tile {
            terrain: terrain,
            belief: Belief::Unknown,
            parent: None,
            g: 0.0,
            h: 0.0,
            visited: false,
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

    pub fn visited(&self) -> bool {
        self.visited
    }

    pub fn visit(&mut self, parent: Point, g: Distance, h: Distance) {
        self.parent = Some(parent);
        self.visited = true;
        self.g = g;
        self.h = h;
    }

    pub fn visit_initial(&mut self, h: Distance) {
        self.parent = None;
        self.visited = true;
        self.g = 0.0;
        self.h = h;
    }

    pub fn unvisit(&mut self) {
        self.visited = false;
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
}

impl Grid {
    pub fn new(tiles: Vec<Vec<Tile>>) -> Grid {
        Grid { tiles: tiles }
    }

    pub fn get(&self, point: &Point) -> Option<&Tile> {
        self.tiles.get(point.y()).and_then(|row| row.get(point.x()))
    }

    pub fn get_mut(&mut self, point: &Point) -> Option<&mut Tile> {
        self.tiles.get_mut(point.y()).and_then(|row| row.get_mut(point.x()))
    }

    /// Marks all cells as unvisited (i.e. not Open or Closed). Must be called
    /// before every search episode.
    pub fn restage(&mut self) {
        for row in self.tiles.iter_mut() {
            for cell in row.iter_mut() {
                cell.unvisit();
            }
        }
    }

    pub fn look(&mut self, point: &Point) {
        for neighbor in point.neighbors().iter() {
            if let Some(ref mut tile) = self.get_mut(neighbor) {
                tile.look();
            }
        }
    }

    pub fn iter(&self) -> Iter<Vec<Tile>> {
        self.tiles.iter()
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
        for row in self.tiles.iter() {
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
    fn octile_distance() {
        let p0 = Point::new(0, 0);
        let p1 = Point::new(0, 1);
        let p2 = Point::new(1, 1);
        let p3 = Point::new(5, 5);

        assert_eq!(Distance::octile(&p0, &p1), 1.0);
        assert_eq!(Distance::octile(&p0, &p2), SQRT_2);
        assert_eq!(Distance::octile(&p1, &p3), 1.0 + 4.0 * SQRT_2);
    }

    #[test]
    fn euclidean_distance() {
        let p0 = Point::new(0, 0);
        let p1 = Point::new(0, 1);
        let p2 = Point::new(1, 1);
        let p3 = Point::new(5, 5);

        assert_eq!(Distance::euclidean(&p0, &p1), 1.0);
        assert_eq!(Distance::euclidean(&p0, &p2), SQRT_2);
        assert_eq!(Distance::euclidean(&p2, &p3), 4.0 * SQRT_2);
    }
}
