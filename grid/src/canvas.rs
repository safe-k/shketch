use std::error;
use std::fmt;
use std::io::{self, Write};
use std::result;

use termion::clear;
use termion::cursor;
use termion::event::MouseEvent;

use crate::path::{self, Connect};
use crate::unit::{self, Erase};

type Result = result::Result<(), Error>;

#[derive(Debug)]
pub enum Error {
    Fmt(fmt::Error),
    Io(io::Error),
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Fmt(e) => Some(e),
            Error::Io(e) => Some(e),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Error::Fmt(_) => "failed to format message to stream",
            Error::Io(_) => "failed to perform I/O operation",
        };

        write!(f, "could not update canvas; {}", msg)
    }
}

impl From<fmt::Error> for Error {
    fn from(fmt_error: fmt::Error) -> Self {
        Error::Fmt(fmt_error)
    }
}

impl From<io::Error> for Error {
    fn from(io_error: io::Error) -> Self {
        Error::Io(io_error)
    }
}

pub enum Style {
    Plot,
    Line,
}

impl From<char> for Style {
    fn from(char: char) -> Self {
        match char {
            '2' => Style::Line,
            _ => Style::Plot,
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Style::Plot
    }
}

pub struct Canvas<W, B>
where
    W: Write,
    B: Connect,
{
    writer: W,
    brush: B,
    style: Style,
    base: Vec<unit::Segment>,
    overlay: unit::Segment,
    sketch: unit::Segment,
    cursor: path::Point,
}

impl<W, B> Canvas<W, B>
where
    W: Write,
    B: Connect,
{
    const TOOLBAR_BOUNDARY: u16 = 3;

    pub fn new(writer: W, brush: B) -> Self {
        Self {
            writer,
            brush,
            style: Default::default(),
            base: Default::default(),
            overlay: Default::default(),
            sketch: Default::default(),
            cursor: Default::default(),
        }
    }

    pub fn init(&mut self) -> Result {
        write!(self.writer, "{}{}", clear::All, cursor::Hide)?;
        self.writer.flush()?;
        Ok(())
    }

    pub fn pin(&mut self, overlay: unit::Segment) {
        self.overlay = overlay;
    }

    pub fn alt_style(&mut self, style: Style) {
        self.style = style;
    }

    pub fn update(&mut self, mouse_event: MouseEvent) -> Result {
        match mouse_event {
            MouseEvent::Press(_, a, b) => self.cursor.move_to(a, b),
            MouseEvent::Hold(a, b) => {
                // Reserve toolbar space
                if b < Self::TOOLBAR_BOUNDARY {
                    return Ok(());
                }

                match self.style {
                    Style::Plot => {
                        self.sketch += self.brush.connect(self.cursor, path::Point::new(a, b));
                        self.cursor.move_to(a, b);
                    }
                    Style::Line => {
                        self.sketch.erase(&mut self.writer)?;
                        self.sketch = self.brush.connect(self.cursor, path::Point::new(a, b));
                    }
                }
            }
            MouseEvent::Release(_, _) => {
                self.base.push(self.sketch.clone());
                self.sketch.clear();
            }
        }
        Ok(())
    }

    pub fn draw(&mut self) -> Result {
        for segment in &self.base {
            write!(self.writer, "{}", segment)?;
        }
        write!(self.writer, "{}{}", self.sketch, self.overlay)?;
        self.writer.flush()?;
        Ok(())
    }

    pub fn snapshot(&self) -> Vec<unit::Segment> {
        self.base.clone()
    }

    pub fn undo(&mut self) -> Result {
        if let Some(mut segment) = self.base.pop() {
            segment.erase(&mut self.writer)?;
        }
        Ok(())
    }

    pub fn clear(&mut self) -> Result {
        self.base.clear();
        self.sketch.clear();

        write!(
            self.writer,
            "{}{}",
            cursor::Goto(1, Self::TOOLBAR_BOUNDARY),
            clear::All
        )?;
        self.writer.flush()?;
        Ok(())
    }
}

impl<W, B> Drop for Canvas<W, B>
where
    W: Write,
    B: Connect,
{
    fn drop(&mut self) {
        write!(
            self.writer,
            "{}{}{}",
            clear::All,
            cursor::Goto(1, 1),
            cursor::Show
        )
        .expect("Clear canvas before dropping");
    }
}