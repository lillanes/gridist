use std::fmt::Display;
use std::fs::File;
use std::io::{BufReader, Error as IOError, Read};
use std::path::Path;

use grid::{Grid, Terrain, Tile};

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
        while self.position < self.data.len() &&
              !self.data[self.position].is_whitespace() {
            word.push(self.data[self.position]);
            self.shift();
        }
        let word: String = word.into_iter().collect();
        match usize::from_str_radix(&word, 10) {
            Ok(size) => Ok(size),
            Err(_) => {
                Err(self.error(format!("Expected integer, found '{}'.", word)))
            }
        }
    }

    fn parse_grid(&mut self) -> Result<Grid, ParseError> {
        self.consume_word("type")?;
        self.consume_word("octile")?;

        let height = self.parse_constant("height")?;
        let width = self.parse_constant("width")?;

        self.consume_word("map")?;

        let mut tiles = Vec::with_capacity(height);

        for _ in 0..height {
            let mut row = Vec::with_capacity(width);
            self.consume_ws();
            for _ in 0..width {
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
                row.push(Tile::new(value));
                self.shift();
            }
            tiles.push(row);
        }
        Ok(Grid::new(tiles))
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
    let chars =
        chars_from_file(filename).expect(&format!("Could not read from file {}",
                                                  filename));
    grid_from_chars(chars).unwrap_or_else(|e| {
        panic!("Parsing error: {} ({}@{}:{})",
               e.description,
               filename,
               e.line,
               e.column)
    })
}

#[cfg(test)]
pub fn grid_from_str(grid: &str) -> Grid {
    grid_from_chars(grid.chars().collect())
        .unwrap_or_else(|e| {
            panic!("Parsing error: {} ({}:{})",
                   e.description,
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
}
