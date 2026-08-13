use crate::{
    line_map::LineMap,
    location::Offset,
    source::Source,
};

pub struct Manager {
    offset: Offset,
    sources: Vec<Source>,
    lines: LineMap,
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

    fn make_next_offset(&self, source: &Source) -> Offset {
        self.offset + Offset(source.source().len() as u32)
    }

    pub fn add_source(&mut self, mut source: Source) -> &Source {
        source.patch_offset(self.offset);
        self.offset = self.make_next_offset(&source);

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
}
