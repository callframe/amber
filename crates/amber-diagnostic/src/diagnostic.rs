use std::fmt::{
    self,
    Display,
};

use amber_source::location::Location;
use typed_builder::TypedBuilder;

#[derive(Clone, Copy)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Info => write!(f, "info"),
        }
    }
}

#[derive(TypedBuilder)]
pub struct Diagnostic {
    severity: Severity,
    #[builder(setter(into))]
    message: String,
    location: Location,
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
}
