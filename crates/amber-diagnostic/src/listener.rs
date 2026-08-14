use std::io::Write;

use amber_source::manager::Manager;

use crate::diagnostic::Diagnostic;

pub trait Listener {
    fn report(&mut self, diagnostic: Diagnostic);
}

pub struct StreamListener<'life> {
    manager: &'life Manager,
    out: &'life mut dyn Write,
}

impl<'life> StreamListener<'life> {
    pub fn new(manager: &'life Manager, out: &'life mut dyn Write) -> Self {
        StreamListener { manager, out }
    }
}

impl<'life> Listener for StreamListener<'life> {
    fn report(&mut self, diagnostic: Diagnostic) {
        let (lines, source) = match self.manager.lookup_lines(diagnostic.get_location()) {
            Ok(found_lines) => (found_lines.get_lines(), found_lines.get_source()),
            Err(_) => unreachable!("Manager did not find lines for a diagnostic"), // Fail loudly; This might use `log` crate at some point and just return.
        };

        // <severity>: <message>
        writeln!(
            self.out,
            "{}: {}",
            diagnostic.get_severity(),
            diagnostic.get_message()
        )
        .unwrap();

        let first_line = lines.first().unwrap();
        let column = first_line.get_column(diagnostic.get_location());

        // --> <filename>:<line>:<column>
        writeln!(
            self.out,
            "--> {}:{}:{}",
            source.get_name(),
            first_line.get_number(),
            column,
        )
        .unwrap();
    }
}
