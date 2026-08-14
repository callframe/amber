use thiserror::Error;

use crate::location::{
    Location,
    Offset,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Line {
    location: Location,
    line_number: u32,
    source_offset: Offset,
}

impl Line {
    fn new(start_offset: Offset, len: u32, line_number: u32, source_offset: Offset) -> Self {
        Line {
            location: Location::new(start_offset, len),
            line_number,
            source_offset,
        }
    }

    pub fn get_column(&self, location: Location) -> u32 {
        assert!(self.location.get_start() <= location.get_start());
        assert!(self.location.get_end() >= location.get_start());
        location.get_start().0 - self.location.get_start().0
    }

    pub fn get_number(&self) -> u32 {
        self.line_number
    }

    pub fn get_location(&self) -> Location {
        self.location
    }
}

#[derive(Debug, Error)]
pub enum LineMapError {
    #[error("Offset {0} is not within any line")]
    LinesNotFound(Offset),

    #[error("Location {0} crosses file boundaries")]
    CrossesFileBoundary(Location),
}

#[derive(Debug)]
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

    pub(crate) fn extend_from(&mut self, text: &str, offset: Offset) {
        let mut cursor = text;
        let mut line_start = offset;
        let mut line_number = 1;

        while !cursor.is_empty() {
            let consumed = match cursor.find('\n') {
                Some(end) => end + 1,
                None => cursor.len(),
            };

            let line = Line::new(line_start, consumed as u32, line_number, offset);
            self.lines.push(line);

            line_number += 1;
            line_start += consumed as u32;
            cursor = &cursor[consumed..];
        }
    }

    pub fn lookup(&self, location: Location) -> Result<&[Line], LineMapError> {
        let mut start = location.get_start();
        let mut end = location.get_end();

        // An empty location covers no byte, so clamp it onto the preceding one.
        if location.is_empty() {
            start = start.saturating_sub(1);
            end = start + 1;
        }

        let first = self.lines.partition_point(|l| l.location.get_end() <= start);
        let last = self.lines.partition_point(|l| l.location.get_start() < end);

        let lines = &self.lines[first..last];
        let (Some(head), Some(tail)) = (lines.first(), lines.last()) else {
            return Err(LineMapError::LinesNotFound(start));
        };

        if head.source_offset != tail.source_offset {
            return Err(LineMapError::CrossesFileBoundary(location));
        }

        Ok(lines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map_of(texts: &[&str]) -> LineMap {
        let mut map = LineMap::new();
        let mut offset = Offset(0);
        for text in texts {
            map.extend_from(text, offset);
            offset += text.len() as u32;
        }
        map
    }

    fn spans(map: &LineMap) -> Vec<(u32, u32, u32)> {
        map.lines
            .iter()
            .map(|l| (l.location.get_start().0, l.location.get_len(), l.line_number))
            .collect()
    }

    fn numbers(map: &LineMap, location: Location) -> Vec<u32> {
        map.lookup(location).unwrap().iter().map(Line::get_number).collect()
    }

    #[test]
    fn maps_text_onto_lines_that_own_their_newline() {
        let cases = [
            ("", vec![]),
            ("abc", vec![(0, 3, 1)]),
            ("abc\n", vec![(0, 4, 1)]),
            ("abc\ndef", vec![(0, 4, 1), (4, 3, 2)]),
            ("a\n\nb", vec![(0, 2, 1), (2, 1, 2), (3, 1, 3)]),
            ("a\r\nb", vec![(0, 3, 1), (3, 1, 2)]),
        ];

        for (text, expected) in cases {
            assert_eq!(spans(&map_of(&[text])), expected, "mapping {text:?}");
        }
    }

    #[test]
    fn continues_offsets_but_restarts_line_numbers_per_source() {
        assert_eq!(
            spans(&map_of(&["a\nb", "c\nd"])),
            [(0, 2, 1), (2, 1, 2), (3, 2, 1), (5, 1, 2)]
        );
    }

    #[test]
    fn lines_tile_the_source_without_gaps() {
        let text = "abc\n\ndef\r\n\nghi";
        let map = map_of(&[text]);

        let mut expected = Offset(0);
        for line in &map.lines {
            assert_eq!(line.location.get_start(), expected);
            expected = line.location.get_end();
        }
        assert_eq!(expected, Offset(text.len() as u32));
    }

    #[test]
    fn looks_up_the_lines_a_span_covers() {
        let map = map_of(&["abc\ndef\nghi"]);
        let cases = [
            (Location::new(Offset(0), 3), vec![1]),
            (Location::new(Offset(0), 4), vec![1]),
            (Location::new(Offset(4), 3), vec![2]),
            (Location::new(Offset(2), 7), vec![1, 2, 3]),
        ];

        for (location, expected) in cases {
            assert_eq!(numbers(&map, location), expected, "looking up {location}");
        }
    }

    #[test]
    fn looks_up_a_blank_line() {
        let map = map_of(&["a\n\nb"]);
        assert_eq!(numbers(&map, Location::new(Offset(2), 1)), [2]);
    }

    #[test]
    fn clamps_an_empty_location_onto_the_preceding_byte() {
        let map = map_of(&["abc\ndef"]);
        let cases = [(Offset(0), vec![1]), (Offset(4), vec![1]), (Offset(7), vec![2])];

        for (point, expected) in cases {
            let location = Location::new(point, 0);
            assert_eq!(numbers(&map, location), expected, "clamping {location}");
        }
    }

    #[test]
    fn rejects_a_span_crossing_sources() {
        let map = map_of(&["ab", "cd"]);
        let error = map.lookup(Location::new(Offset(1), 2)).unwrap_err();
        assert!(matches!(error, LineMapError::CrossesFileBoundary(_)));
    }

    #[test]
    fn rejects_a_location_outside_the_map() {
        let map = map_of(&["abc"]);
        let empty = map_of(&[]);
        let cases = [
            (&map, Location::new(Offset(3), 1)),
            (&map, Location::new(Offset(9), 0)),
            (&empty, Location::new(Offset(0), 1)),
            (&empty, Location::new(Offset(0), 0)),
        ];

        for (map, location) in cases {
            let error = map.lookup(location).unwrap_err();
            assert!(matches!(error, LineMapError::LinesNotFound(_)), "looking up {location}");
        }
    }
}
