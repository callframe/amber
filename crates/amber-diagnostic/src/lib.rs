use std::fmt::{
    self,
    Display,
};

use amber_source::location::Location;
use typed_builder::TypedBuilder;

pub trait Listener {
    fn emit(&mut self, diagnostic: Diagnostic);
}

#[derive(Debug, Clone, Copy)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        }
    }
}

impl Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, TypedBuilder)]
pub struct Label {
    severity: Severity,
    #[builder(setter(into))]
    message: String,
    location: Location,
}

impl Label {
    pub fn get_severity(&self) -> Severity {
        self.severity
    }

    pub fn get_message(&self) -> &str {
        &self.message
    }

    pub fn get_location(&self) -> Location {
        self.location
    }
}

#[derive(Debug, TypedBuilder)]
pub struct Diagnostic {
    severity: Severity,
    #[builder(setter(into))]
    message: String,
    location: Location,
    #[builder(default)]
    labels: Vec<Label>,
}

impl Diagnostic {
    pub fn get_severity(&self) -> Severity {
        self.severity
    }

    pub fn get_message(&self) -> &str {
        &self.message
    }

    pub fn get_location(&self) -> Location {
        self.location
    }

    pub fn get_labels(&self) -> &[Label] {
        &self.labels
    }
}
