use std::fs::File;

use amber_source::{
    location::Location,
    manager::Manager,
    source::Source,
};
use mimalloc::MiMalloc;

use crate::cli::Cli;
use clap::Parser;

mod cli;

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

        println!("Source name: {}", source.name());
        println!("Source offset: {}", source.offset());
    }

    let lines = manager
        .get_lines()
        .lookup(Location::new(8, 20))
        .expect("Failed to get lines");

    for line in lines {
        println!(
            "Line {}: start: {:?}, end: {:?}",
            line.get_line_number(),
            line.get_location().start(),
            line.get_location().end()
        );
    }
}
