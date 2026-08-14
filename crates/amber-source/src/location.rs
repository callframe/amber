use std::{
    cmp::Ordering,
    fmt::{
        self,
        Display,
    },
    ops::{
        Add,
        Sub,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Offset(pub u32);

impl Add for Offset {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Offset(self.0 + other.0)
    }
}

impl Add<u32> for Offset {
    type Output = Self;

    fn add(self, other: u32) -> Self {
        Offset(self.0 + other)
    }
}

impl Sub for Offset {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Offset(self.0 - other.0)
    }
}

impl PartialEq<u32> for Offset {
    fn eq(&self, other: &u32) -> bool {
        self.0 == *other
    }
}

impl PartialOrd<u32> for Offset {
    fn partial_cmp(&self, other: &u32) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialEq<Offset> for u32 {
    fn eq(&self, other: &Offset) -> bool {
        *self == other.0
    }
}

impl PartialOrd<Offset> for u32 {
    fn partial_cmp(&self, other: &Offset) -> Option<Ordering> {
        self.partial_cmp(&other.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    start: Offset,
    end: Offset,
}

impl Location {
    pub fn new(start: Offset, end: Offset) -> Self {
        Location { start, end }
    }

    pub fn start(&self) -> Offset {
        self.start
    }

    pub fn end(&self) -> Offset {
        self.end
    }
}

impl PartialEq<u32> for Location {
    fn eq(&self, other: &u32) -> bool {
        self.start.0 <= *other && *other < self.end.0
    }
}

impl PartialOrd<u32> for Location {
    fn partial_cmp(&self, other: &u32) -> Option<Ordering> {
        if self.start.0 > *other {
            Some(Ordering::Greater)
        } else if self.end.0 <= *other {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Equal)
        }
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start.0, self.end.0)
    }
}
