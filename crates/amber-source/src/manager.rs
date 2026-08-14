use thiserror::Error;

use anyhow::Result;

use crate::{
    line_map::{
        Line,
        LineMap,
    },
    location::{
        Location,
        Offset,
    },
    source::Source,
};

#[derive(Debug, Error)]
pub enum ManagerError {
    #[error("Source not found for location: {0}")]
    SourceNotFound(Location),
}

#[derive(Debug, Clone, Copy)]
pub struct FoundLines<'life> {
    lines: &'life [Line],
    source: &'life Source,
}

impl<'life> FoundLines<'life> {
    pub fn new(lines: &'life [Line], source: &'life Source) -> Self {
        FoundLines { lines, source }
    }

    pub fn get_lines(&self) -> &'life [Line] {
        self.lines
    }

    pub fn get_source(&self) -> &'life Source {
        self.source
    }
}

#[derive(Debug)]
pub struct Manager {
    offset: Offset,
    sources: Vec<Source>,
    lines: LineMap,
}

impl Default for Manager {
    fn default() -> Self {
        Self::new()
    }
}

impl Manager {
    pub fn new() -> Self {
        Manager {
            offset: Offset(0),
            sources: Vec::new(),
            lines: LineMap::new(),
        }
    }

    pub fn get_lines(&self) -> &LineMap {
        &self.lines
    }

    fn get_next_offset(&self, source: &Source) -> Offset {
        self.offset + Offset(source.get_source().len() as u32)
    }

    pub fn add_source(&mut self, mut source: Source) -> &Source {
        source.patch_offset(self.offset);
        self.offset = self.get_next_offset(&source);

        let source = {
            self.sources.push(source);
            self.sources.last().unwrap()
        };

        self.lines.extend_from(source.get_source(), source.get_offset());
        source
    }

    pub fn lookup(&self, offset: Offset) -> Option<&Source> {
        self.sources
            .binary_search_by(|s| s.partial_cmp(&offset).unwrap())
            .ok()
            .map(|i| &self.sources[i])
    }

    pub fn lookup_lines<'life>(&'life self, location: Location) -> Result<FoundLines<'life>> {
        let source = match self.lookup(location.get_start()) {
            Some(source) => source,
            None => return Err(ManagerError::SourceNotFound(location).into()),
        };

        let lines = self.lines.lookup(location)?;
        Ok(FoundLines::new(lines, source))
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::NamedTempFile;

    use super::*;
    use crate::line_map::LineMapError;

    struct TestSource {
        _file: NamedTempFile,
        source: Source,
    }

    fn source_from(name: &str, text: &str) -> TestSource {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(text.as_bytes()).unwrap();
        file.flush().unwrap();

        let source = Source::new(name, file.as_file()).unwrap();
        TestSource { _file: file, source }
    }

    #[test]
    fn assigns_offsets_sequentially() {
        let first = source_from("first", "abc");
        let second = source_from("second", "de");

        let mut manager = Manager::new();
        assert_eq!(manager.add_source(first.source).get_offset(), Offset(0));
        assert_eq!(manager.add_source(second.source).get_offset(), Offset(3));
    }

    #[test]
    fn finds_the_source_owning_an_offset() {
        let first = source_from("first", "abc");
        let second = source_from("second", "de");

        let mut manager = Manager::new();
        manager.add_source(first.source);
        manager.add_source(second.source);

        let name_at = |offset| manager.lookup(offset).map(Source::get_name);

        assert_eq!(name_at(Offset(0)), Some("first"));
        assert_eq!(name_at(Offset(2)), Some("first"));
        assert_eq!(name_at(Offset(3)), Some("second"));
        assert_eq!(name_at(Offset(4)), Some("second"));
        assert_eq!(name_at(Offset(5)), None);
    }

    #[test]
    fn looks_up_lines_together_with_their_source() {
        let first = source_from("first", "abc\ndef");
        let second = source_from("second", "ghi");

        let mut manager = Manager::new();
        manager.add_source(first.source);
        manager.add_source(second.source);

        let found = manager.lookup_lines(Location::new(Offset(7), 3)).unwrap();

        assert_eq!(found.get_source().get_name(), "second");
        assert_eq!(found.get_lines().len(), 1);
        assert_eq!(found.get_lines()[0].get_number(), 1);
    }

    #[test]
    fn looks_up_a_span_ending_on_the_last_byte_of_a_source() {
        let first = source_from("first", "abc\ndef");
        let second = source_from("second", "ghi");

        let mut manager = Manager::new();
        manager.add_source(first.source);
        manager.add_source(second.source);

        let found = manager.lookup_lines(Location::new(Offset(4), 3)).unwrap();

        assert_eq!(found.get_source().get_name(), "first");
        assert_eq!(found.get_lines()[0].get_number(), 2);
    }

    #[test]
    fn rejects_a_span_crossing_two_sources() {
        let first = source_from("first", "abc");
        let second = source_from("second", "def");

        let mut manager = Manager::new();
        manager.add_source(first.source);
        manager.add_source(second.source);

        let Err(error) = manager.lookup_lines(Location::new(Offset(2), 2)) else {
            panic!("expected the lookup to fail");
        };

        assert!(matches!(
            error.downcast_ref::<LineMapError>(),
            Some(LineMapError::CrossesFileBoundary(_))
        ));
    }

    #[test]
    fn rejects_a_location_outside_every_source() {
        let first = source_from("first", "abc");

        let mut manager = Manager::new();
        manager.add_source(first.source);

        let Err(error) = manager.lookup_lines(Location::new(Offset(9), 1)) else {
            panic!("expected the lookup to fail");
        };

        assert!(matches!(
            error.downcast_ref::<ManagerError>(),
            Some(ManagerError::SourceNotFound(_))
        ));
    }
}
