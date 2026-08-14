use std::{
    cmp::Ordering,
    fs::File,
    path::Iter,
};

use anyhow::Result;
use memmap2::Mmap;
use std::str;

use crate::location::Offset;

#[derive(Debug)]
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

    // Should not fail if there is no bug
    pub fn get_offset(&self) -> Offset {
        match self.offset {
            Some(offset) => offset,
            None => unreachable!("Manager did not patch the source with an offset"),
        }
    }

    pub fn get_end_offset(&self) -> Offset {
        self.get_offset() + self.source.len() as u32
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_source(&self) -> &str {
        match str::from_utf8(&self.source) {
            Ok(s) => s,
            Err(_) => unreachable!("Source was checked for UTF-8 validity during construction"),
        }
    }

    pub(crate) fn patch_offset(&mut self, offset: Offset) {
        self.offset = Some(offset);
    }
}

impl PartialEq<Offset> for Source {
    fn eq(&self, other: &Offset) -> bool {
        self.get_offset() <= other.0 && other.0 < self.get_end_offset()
    }
}

impl PartialOrd<Offset> for Source {
    fn partial_cmp(&self, other: &Offset) -> Option<Ordering> {
        if self.get_offset() > other.0 {
            Some(Ordering::Greater)
        } else if self.get_end_offset() <= other.0 {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Equal)
        }
    }
}
