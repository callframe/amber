use std::cmp::Ordering;

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
    line_no: u32,
}

impl Line {
    fn new(start_offset: u32, end_offset: u32, line_no: u32) -> Self {
        Line {
            location: Location::new(start_offset, end_offset),
            line_no,
        }
    }

    pub fn get_column(&self, location: Location) -> u32 {
        assert!(self.location.start() <= location.start());
        assert!(self.location.end() >= location.end());
        location.start().0 - self.location.start().0
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

pub struct LineMap {
    lines: Vec<Line>,
}

impl LineMap {
    pub fn new() -> Self {
        LineMap { lines: Vec::new() }
    }

    pub(crate) fn extend_from(&mut self, source: &Source) {
        let mut cursor = source.source();
        let mut line_start = source.offset();
        let mut line_no = 1;

        while !cursor.is_empty() {
            let line = match cursor.find('\n') {
                Some(end) => &cursor[..=end],
                None => cursor,
            };

            let line_end = line_start + line.len() as u32;
            self.lines.push(Line::new(line_start, line_end, line_no));

            line_no += 1;
            line_start = line_end;
            cursor = &cursor[line.len()..];
        }
    }

    fn get_line(&self, location: Offset) -> Option<usize> {
        self.lines
            .binary_search_by(|l| l.partial_cmp(&location).unwrap())
            .ok()
            .map(|i| i)
    }

    pub fn get_lines(&self, location: Location) -> Option<&[Line]> {
        let start_index = self.get_line(location.start())?;
        let end_index = self.get_line(location.end())?;

        Some(&self.lines[start_index..=end_index])
    }
}
