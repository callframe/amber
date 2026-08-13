use std::fs::File;

use amber_source::{
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

    let mut manager = Manager::new();
    let source = {
        let source = Source::new(cli.source(), &file).expect("Failed to create source");
        manager.add_source(source)
    };

    println!("Source name: {}", source.name());
    println!("Source offset: {}", source.offset());

    
}
