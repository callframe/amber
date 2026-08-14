use std::{
    fmt::{
        self,
        Debug,
    },
    io::Write,
};

use amber_source::{
    location::Location,
    manager::Manager,
};

use crate::{
    Listener,
    diagnostic::{
        Diagnostic,
        Severity,
    },
};

pub struct StreamListener<'life> {
    manager: &'life Manager,
    writer: &'life mut dyn Write,
}

impl<'life> StreamListener<'life> {
    pub fn new(manager: &'life Manager, writer: &'life mut dyn Write) -> Self {
        StreamListener { manager, writer }
    }

    fn write_entry(&mut self, severity: Severity, message: &str, location: Location, indent: bool) {
        let found_lines = match self.manager.lookup_lines(location) {
            Ok(found_lines) => found_lines,
            Err(error) => panic!("Cannot report diagnostic at {location}: {error}"),
        };

        let line = match found_lines.get_lines().first() {
            Some(line) => line,
            None => unreachable!("Manager returned no lines for location {location}"),
        };

        let name = found_lines.get_source().get_name();
        let number = line.get_number();

        // Columns are counted from zero internally, but reported from one.
        let column = line.get_column(location) + 1;
        let indent = if indent { "    " } else { "" };

        writeln!(self.writer, "{indent}{name}:{number}:{column}: {severity}: {message}")
            .expect("Failed to write diagnostic");
    }
}

impl<'life> Listener for StreamListener<'life> {
    fn report(&mut self, diagnostic: Diagnostic) {
        self.write_entry(
            diagnostic.get_severity(),
            diagnostic.get_message(),
            diagnostic.get_location(),
            false,
        );

        for label in diagnostic.get_labels() {
            self.write_entry(label.get_severity(), label.get_message(), label.get_location(), true);
        }
    }
}

impl<'life> Debug for StreamListener<'life> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StreamListener")
            .field("manager", &self.manager)
            .field("writer", &"<dyn Write>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use amber_source::{
        location::Offset,
        source::Source,
    };
    use tempfile::NamedTempFile;

    use super::*;
    use crate::diagnostic::Label;

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

    fn report_all(manager: &Manager, diagnostics: Vec<Diagnostic>) -> String {
        let mut buffer = Vec::new();
        {
            let mut listener = StreamListener::new(manager, &mut buffer);
            for diagnostic in diagnostics {
                listener.report(diagnostic);
            }
        }
        String::from_utf8(buffer).unwrap()
    }

    fn error_at(message: &str, start: u32, len: u32) -> Diagnostic {
        Diagnostic::builder()
            .severity(Severity::Error)
            .message(message)
            .location(Location::new(Offset(start), len))
            .build()
    }

    #[test]
    fn reports_a_diagnostic_without_labels() {
        let text = source_from("main.ab", "let foo = 1\n");

        let mut manager = Manager::new();
        manager.add_source(text.source);

        let output = report_all(&manager, vec![error_at("undefined variable", 4, 3)]);
        assert_eq!(output, "main.ab:1:5: error: undefined variable\n");
    }

    #[test]
    fn indents_labels_under_their_diagnostic() {
        let text = source_from("main.ab", "let foo = 1\nlet bar = 2\n");

        let mut manager = Manager::new();
        manager.add_source(text.source);

        let diagnostic = Diagnostic::builder()
            .severity(Severity::Error)
            .message("undefined variable")
            .location(Location::new(Offset(16), 3))
            .labels(vec![
                Label::builder()
                    .severity(Severity::Info)
                    .message("declared here")
                    .location(Location::new(Offset(4), 3))
                    .build(),
                Label::builder()
                    .severity(Severity::Warning)
                    .message("shadowed here")
                    .location(Location::new(Offset(0), 3))
                    .build(),
            ])
            .build();

        let output = report_all(&manager, vec![diagnostic]);
        assert_eq!(
            output,
            concat!(
                "main.ab:2:5: error: undefined variable\n",
                "    main.ab:1:5: info: declared here\n",
                "    main.ab:1:1: warning: shadowed here\n",
            )
        );
    }

    #[test]
    #[should_panic(expected = "Cannot report diagnostic")]
    fn panics_when_the_location_has_no_source() {
        let manager = Manager::new();
        report_all(&manager, vec![error_at("undefined variable", 0, 3)]);
    }
}
