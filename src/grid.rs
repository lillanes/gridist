use std::cmp::{max, min};
use std::f64::consts::SQRT_2;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::fs::File;
use std::io::{BufReader, Error as IOError, Read};
use std::path::Path;

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
    fn passable(&self) -> Belief {
        match *self {
            Terrain::Ground => Belief::Passable,
            _ => Belief::Impassable,
        }
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
pub struct Cell {
    terrain: Terrain,
    belief: Belief,
    h: Distance,
    g: Distance,
    visited: bool,
}

impl Cell {
    pub fn new(terrain: Terrain) -> Cell {
        Cell {
            terrain: terrain,
            belief: Belief::Unknown,
            h: 0.0,
            g: 0.0,
            visited: false,
        }
    }

    pub fn look(&mut self) {
        if self.belief == Belief::Unknown {
            self.belief = self.terrain.passable();
        }
    }

    pub fn passable(&self) -> bool {
        self.belief != Belief::Impassable
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.terrain)
    }
}

#[derive(Debug)]
pub struct Grid {
    cells: Vec<Vec<Cell>>,
    height: usize,
    width: usize,
}

impl Grid {
    pub fn height(&self) -> usize {
        self.height
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn get(&self, point: &Point) -> Option<&Cell> {
        self.cells.get(point.y()).and_then(|row| row.get(point.x()))
    }
}

impl Display for Grid {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        for row in self.cells.iter() {
            for cell in row.iter() {
                write!(f, "{}", cell)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct ParseError {
    line: usize,
    column: usize,
    description: String,
}

#[derive(Debug)]
struct Parser {
    data: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Parser {
    fn new(data: Vec<char>) -> Parser {
        Parser {
            data: data,
            position: 0,
            line: 0,
            column: 0,
        }
    }

    fn shift(&mut self) {
        if self.position < self.data.len() {
            if self.data[self.position] == '\n' {
                self.line += 1;
                self.column = 0;
            } else {
                self.column += 1;
            }
            self.position += 1;
        }
    }

    fn consume_word(&mut self, word: &str) -> Result<(), ParseError> {
        self.consume_ws();

        let mut read = Vec::new();
        while self.position < self.data.len() {
            let c = self.data[self.position];
            if !c.is_whitespace() {
                read.push(c);
                self.shift();
            } else {
                break;
            }
        }

        let read: String = read.into_iter().collect();
        if read == word {
            Ok(())
        } else {
            Err(self.error(format!("Expected '{}', found '{}'.", word, read)))
        }
    }

    fn consume_ws(&mut self) {
        while self.position < self.data.len() {
            let c = self.data[self.position];
            if c.is_whitespace() {
                self.shift();
            } else {
                break;
            }
        }
    }

    fn error(&self, description: String) -> ParseError {
        ParseError {
            line: self.line,
            column: self.column,
            description: description,
        }
    }

    fn parse_constant(&mut self, name: &str) -> Result<usize, ParseError> {
        self.consume_ws();
        self.consume_word(name)?;

        self.consume_ws();
        self.parse_int()
    }

    fn parse_int(&mut self) -> Result<usize, ParseError> {
        let mut word = Vec::new();
        while self.position < self.data.len() && !self.data[self.position].is_whitespace() {
            word.push(self.data[self.position]);
            self.shift();
        }
        let word: String = word.into_iter().collect();
        match usize::from_str_radix(&word, 10) {
            Ok(size) => Ok(size),
            Err(_) => Err(self.error(format!("Expected integer, found '{}'.", word))),
        }
    }

    fn parse_grid(&mut self) -> Result<Grid, ParseError> {
        self.consume_word("type")?;
        self.consume_word("octile")?;

        let height = self.parse_constant("height")?;
        let width = self.parse_constant("width")?;

        self.consume_word("map")?;

        let mut cells = Vec::with_capacity(height);

        for y in 0..height {
            let mut row = Vec::with_capacity(width);
            self.consume_ws();
            for x in 0..width {
                let value = match self.data[self.position] {
                    '.' => Terrain::Ground,
                    'G' => Terrain::Ground,
                    '@' => Terrain::OutOfBounds,
                    'O' => Terrain::OutOfBounds,
                    'T' => Terrain::Trees,
                    'S' => Terrain::Swamp,
                    'W' => Terrain::Water,
                    '\n' => {
                        let message = format!("Unexpected end of line.");
                        return Err(self.error(message));
                    }
                    other @ _ => {
                        let message = format!("Unrecognized symbol: {}", other);
                        return Err(self.error(message));
                    }
                };
                row.push(Cell::new(value));
                self.shift();
            }
            cells.push(row);
        }
        Ok(Grid {
               cells: cells,
               height: height,
               width: width,
           })
    }
}

fn grid_from_chars(data: Vec<char>) -> Result<Grid, ParseError> {
    let mut parser = Parser::new(data);

    parser.parse_grid()
}

fn chars_from_file<P>(filename: &P) -> Result<Vec<char>, IOError>
    where P: AsRef<Path> + ?Sized
{
    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);
    let mut buffer = String::new();
    reader.read_to_string(&mut buffer)?;
    Ok(buffer.chars().collect())
}

/// Builds a `Grid` out of all the (first) map in the file pointed at by
/// `filename`.
///
/// The file should be in the format specified in
/// http://movingai.com/benchmarks/formats.html
pub fn grid_from_file<P>(filename: &P) -> Grid
    where P: AsRef<Path> + Display + ?Sized
{
    let chars = chars_from_file(filename).expect(&format!("Could not read from file {}", filename));
    grid_from_chars(chars).unwrap_or_else(|e| {
        panic!("Parsing error: {} ({}@{}:{})",
               e.description,
               filename,
               e.line,
               e.column)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD_MAP: &str = "type octile
height 2
width 2
map
T.
.T";
    const PASSABLE: Point = Point { y: 0, x: 1 };
    const IMPASSABLE: Point = Point { y: 1, x: 1 };
    const OUTSIDE: Point = Point { y: 2, x: 2 };

    const BAD_MAP: &str = "type octile
height 2
width 2
map
Tf
.T";

    #[test]
    fn read_grid_from_chars() {
        let grid = grid_from_chars(GOOD_MAP.chars().collect()).unwrap();
        println!("Grid:\n{}", grid);
    }

    #[test]
    #[should_panic]
    fn read_unreadable_map_from_chars() {
        grid_from_chars(BAD_MAP.chars().collect()).unwrap();
    }

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
