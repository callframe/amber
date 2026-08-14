use std::{
    fs::File,
    io,
};

use amber_parser::scanner::Scanner;
use amber_source::{
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

    let source_offset = {
        let source = Source::new(cli.source(), &file).expect("Failed to create source");
        manager.add_source(source)
    };

    let mut stdout = io::stdout().lock();
    let mut listener = ConsoleListener::new(&manager, &mut stdout);

    let source = manager.lookup(source_offset).expect("Failed to lookup source");
    let scanner = Scanner::new(source, &mut listener);
    for _ in scanner {}
}
