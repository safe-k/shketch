use std::cmp;
use std::fmt;
use std::io::Write;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Point {
    x: u16,
    y: u16,
}

impl Point {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    pub fn x(self) -> u16 {
        self.x
    }

    pub fn y(self) -> u16 {
        self.y
    }

    pub fn move_to(&mut self, x: u16, y: u16) {
        self.x = x;
        self.y = y;
    }

    pub fn move_up(&mut self) {
        self.y -= 1;
    }

    pub fn move_down(&mut self) {
        self.y += 1;
    }

    pub fn move_left(&mut self) {
        self.x -= 1;
    }

    pub fn move_right(&mut self) {
        self.x += 1;
    }
}

impl Default for Point {
    fn default() -> Self {
        Self { x: 1, y: 1 }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Cell {
    pos: Point,
    content: char,
}

impl Cell {
    pub fn new(pos: Point, content: char) -> Self {
        Self { pos, content }
    }

    pub fn pos(&self) -> &Point {
        &self.pos
    }

    pub fn content(self) -> char {
        self.content
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\x1B[{};{}H{}", self.pos.y, self.pos.x, self.content)
    }
}

pub fn clear_cell<W: Write>(mut cell: Cell, writer: &mut W) {
    cell.content = ' ';
    write!(writer, "{}", cell).unwrap();
}

#[derive(Debug, Default, Clone)]
pub struct Segment {
    cells: Vec<Cell>,
}

impl Segment {
    pub fn new() -> Self {
        Self { cells: Vec::new() }
    }

    pub fn from_str(start: Point, str: &str) -> Self {
        let mut cells = Vec::new();
        let mut cursor = start;
        for char in str.as_bytes() {
            cells.push(Cell::new(cursor, (*char) as char));
            cursor.move_right();
        }

        Self { cells }
    }

    pub fn add(&mut self, cell: Cell) {
        self.cells.push(cell);
    }

    pub fn clear(&mut self) {
        self.cells.clear();
    }
}

impl From<Vec<Cell>> for Segment {
    fn from(cells: Vec<Cell>) -> Self {
        Self { cells }
    }
}

impl From<Segment> for Vec<Cell> {
    fn from(segment: Segment) -> Self {
        segment.cells
    }
}

impl<'a> std::iter::Sum<&'a Segment> for Segment {
    fn sum<I: Iterator<Item=&'a Segment>>(iter: I) -> Self {
        let mut result = Segment::new();
        for segment in iter {
            result += segment.clone()
        }

        result
    }
}

impl std::ops::AddAssign for Segment {
    fn add_assign(&mut self, mut rhs: Self) {
        self.cells.append(rhs.cells.as_mut())
    }
}

impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for cell in &self.cells {
            write!(f, "{}", cell)?;
        }
        Ok(())
    }
}

pub fn clear_segment<W: Write>(segment: Segment, writer: &mut W) {
    for cell in segment.cells {
        clear_cell(cell, writer);
    }
}

#[derive(Debug)]
pub struct CharSet {
    stationary: char,
    up: char,
    down: char,
    left: char,
    right: char,
    diagonal_back: char,
    diagonal_forward: char,
}

impl CharSet {
    pub fn next(&self, from: Point, to: Point) -> char {
        match to {
            Point { x, y } if from.x == x && from.y < y => self.up,
            Point { x, y } if from.x == x && from.y > y => self.down,
            Point { x, y } if from.x < x && from.y == y => self.left,
            Point { x, y } if from.x > x && from.y == y => self.right,
            Point { x, y } if (from.x > x && from.y > y) || (from.x < x && from.y < y) => {
                self.diagonal_back
            }
            Point { x, y } if (from.x > x && from.y < y) || (from.x < x && from.y > y) => {
                self.diagonal_forward
            }
            _ => self.stationary,
        }
    }
}

impl Default for CharSet {
    fn default() -> Self {
        Self {
            stationary: '.',
            up: '|',
            down: '|',
            left: '_',
            right: '_',
            diagonal_back: '\\',
            diagonal_forward: '/',
        }
    }
}

pub trait Connect {
    fn connect(&self, from: Point, to: Point) -> Segment;
}

pub struct Tracer {
    char_set: CharSet,
}

impl Connect for Tracer {
    fn connect(&self, from: Point, to: Point) -> Segment {
        let mut segment = Segment::new();
        let mut cursor = from;

        while cursor != to {
            let current_pos = cursor;

            match cursor.y.cmp(&to.y) {
                cmp::Ordering::Greater => cursor.move_up(),
                cmp::Ordering::Less => cursor.move_down(),
                _ => {},
            };

            match cursor.x.cmp(&to.x) {
                cmp::Ordering::Greater => cursor.move_left(),
                cmp::Ordering::Less => cursor.move_right(),
                _ => {},
            };

            segment.add(Cell::new(cursor, self.char_set.next(current_pos, cursor)));
        }

        segment
    }
}

impl Default for Tracer {
    fn default() -> Self {
        Self {
            char_set: CharSet::default(),
        }
    }
}
