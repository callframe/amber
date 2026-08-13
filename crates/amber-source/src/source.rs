use std::{
    cmp::Ordering,
    fs::File,
};

use anyhow::Result;
use memmap2::Mmap;
use std::str;

use crate::location::Offset;

pub struct Source {
    offset: Option<Offset>,
    name: String,
    // Memory-mapping avoids copying the entire file into a separate buffer.
    // The kernel manages the mapped pages and loads or evicts them as needed.
    source: Mmap,
}

impl Source {
    pub fn new(name: &str, file: &File) -> Result<Self> {
        let text = unsafe { Mmap::map(file)? };
        let _ = str::from_utf8(&text)?;

        let source = Source {
            offset: None,
            name: name.to_string(),
            source: text,
        };

        Ok(source)
    }

    #[cfg(test)]
    pub fn from_str(name: &str, text: &str) -> Result<Self> {
        use memmap2::MmapMut;

        let mut buf = MmapMut::map_anon(text.len())?;
        buf.copy_from_slice(text.as_bytes());

        let source = Source {
            offset: None,
            name: name.to_string(),
            source: buf.make_read_only()?,
        };
        Ok(source)
    }

    // Should not fail if there is no bug
    pub fn offset(&self) -> u32 {
        match self.offset {
            Some(offset) => offset.0,
            None => unreachable!("Manager did not patch the source with an offset"),
        }
    }

    pub fn end_offset(&self) -> u32 {
        self.offset() + self.source().len() as u32
    }

    pub(crate) fn patch_offset(&mut self, offset: Offset) {
        self.offset = Some(offset);
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn source(&self) -> &str {
        match str::from_utf8(&self.source) {
            Ok(s) => s,
            Err(_) => unreachable!("Source was checked for UTF-8 validity during construction"),
        }
    }
}

impl PartialEq<Offset> for Source {
    fn eq(&self, other: &Offset) -> bool {
        self.offset() <= other.0 && other.0 < self.end_offset()
    }
}

impl PartialOrd<Offset> for Source {
    fn partial_cmp(&self, other: &Offset) -> Option<Ordering> {
        if self.offset() > other.0 {
            Some(Ordering::Greater)
        } else if self.end_offset() <= other.0 {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Equal)
        }
    }
}
