use std::cmp::Ordering;

use thiserror::Error;

use crate::{
    location::{
        Location,
        Offset,
    },
    source::Source,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Line {
    location: Location,
    line_number: u32,
    source_offset: Offset,
}

impl Line {
    fn new(
        start_offset: Offset,
        end_offset: Offset,
        line_number: u32,
        source_offset: Offset,
    ) -> Self {
        Line {
            location: Location::new(start_offset, end_offset),
            line_number,
            source_offset,
        }
    }

    pub fn get_column(&self, location: Location) -> u32 {
        assert!(self.location.start() <= location.start());
        assert!(self.location.end() >= location.end());
        location.start().0 - self.location.start().0
    }

    pub fn get_line_number(&self) -> u32 {
        self.line_number
    }

    pub fn get_location(&self) -> Location {
        self.location
    }
}

impl PartialEq<Offset> for Line {
    fn eq(&self, other: &Offset) -> bool {
        self.location.start() <= other.0 && other.0 < self.location.end()
    }
}

impl PartialOrd<Offset> for Line {
    fn partial_cmp(&self, other: &Offset) -> Option<Ordering> {
        if self.location.end() <= other.0 {
            Some(Ordering::Less)
        } else if self.location.start() > other.0 {
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Equal)
        }
    }
}

#[derive(Debug, Error)]
pub enum LineMapError {
    #[error("Location {0} is not within any line")]
    LinesNotFound(Location),

    #[error("Location {0} crosses file boundaries")]
    CrossesFileBoundary(Location),
}

pub struct LineMap {
    lines: Vec<Line>,
}

impl Default for LineMap {
    fn default() -> Self {
        Self::new()
    }
}

impl LineMap {
    pub fn new() -> Self {
        LineMap { lines: Vec::new() }
    }

    pub(crate) fn extend_from(&mut self, source: &Source) {
        let mut cursor = source.source();
        let mut line_start = source.offset();
        let mut line_number = 1;

        while !cursor.is_empty() {
            let line = match cursor.find('\n') {
                Some(end) => &cursor[..=end],
                None => cursor,
            };

            let line_end = line_start + line.len() as u32;
            self.lines.push(Line::new(
                line_start,
                line_end,
                line_number,
                source.offset(),
            ));

            line_number += 1;
            line_start = line_end;
            cursor = &cursor[line.len()..];
        }
    }

    fn get_line(&self, location: Offset) -> Option<(usize, Offset)> {
        self.lines
            .binary_search_by(|l| l.partial_cmp(&location).unwrap())
            .ok()
            .map(|i| (i, self.lines[i].source_offset))
    }

    pub fn lookup(&self, location: Location) -> Result<&[Line], LineMapError> {
        let (start_index, start_source_offset) = self
            .get_line(location.start())
            .ok_or(LineMapError::LinesNotFound(location))?;

        let (end_index, end_source_offset) = self
            .get_line(location.end())
            .ok_or(LineMapError::LinesNotFound(location))?;

        if start_source_offset != end_source_offset {
            return Err(LineMapError::CrossesFileBoundary(location));
        }

        Ok(&self.lines[start_index..=end_index])
    }
}
