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

        self.lines.extend_from(source);
        source
    }

    pub fn get_by_offset(&self, offset: Offset) -> Option<&Source> {
        self.sources
            .binary_search_by(|s| s.partial_cmp(&offset).unwrap())
            .ok()
            .map(|i| &self.sources[i])
    }

    pub fn lookup_lines<'life>(&'life self, location: Location) -> Result<FoundLines<'life>> {
        let source = match self.get_by_offset(location.get_start()) {
            Some(source) => source,
            None => return Err(ManagerError::SourceNotFound(location).into()),
        };

        let lines = self.lines.lookup(location)?;
        Ok(FoundLines::new(lines, source))
    }
}
