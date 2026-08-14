use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
#[command(author, version, about)]
pub struct Cli {
    source: PathBuf, // TODO: Allow stdin
}

impl Cli {
    pub fn source(&self) -> &str {
        self.source
            .to_str()
            .expect("Source path is not valid UTF-8")
    }
}
