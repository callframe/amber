use std::{
    fs::File,
    io,
};

use amber_diagnostic::{
    Diagnostic,
    Label,
    Listener,
    Severity,
};
use amber_source::{
    location::{
        Location,
        Offset,
    },
    manager::Manager,
    source::Source,
};
use mimalloc::MiMalloc;

use crate::{
    cli::Cli,
    console_listener::ConsoleListener,
};
use clap::Parser;

mod cli;
mod console_listener;

#[global_allocator]
static ALLOC: MiMalloc = MiMalloc;

fn main() {
    let cli = Cli::parse();
    let file = File::open(cli.source()).expect("Failed to open source file");

    let mut manager = Manager::default();
    {
        let source = {
            let source = Source::new(cli.source(), &file).expect("Failed to create source");
            manager.add_source(source)
        };

        println!("Source name: {}", source.get_name());
        println!("Source offset: {:?}", source.get_offset());
    }

    let found_lines = manager
        .lookup_lines(Location::new(Offset(8), 15))
        .unwrap_or_else(|e| panic!("{e}"));

    for line in found_lines.get_lines() {
        println!("Line {}: {}", line.get_number(), line.get_location());
    }

    let mut stdout = io::stdout().lock();
    let mut stdout_listener = ConsoleListener::new(&manager, &mut stdout);
    stdout_listener.report(
        Diagnostic::builder()
            .severity(Severity::Error)
            .message("This is a test diagnostic")
            .location(Location::new(Offset(8), 15))
            .labels(vec![
                Label::builder()
                    .severity(Severity::Info)
                    .message("This is a test label")
                    .location(Location::new(Offset(0), 4))
                    .build(),
            ])
            .build(),
    );
}
