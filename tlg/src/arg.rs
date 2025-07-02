use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about)]
/// A command line tool to process TLG files.
pub struct Arg {
    /// Path to the input TLG/PNG file.
    pub input: String,
    /// Path to the output TLG/PNG file.
    pub output: Option<String>,
}

impl Arg {
    pub fn parse() -> Self {
        Parser::parse()
    }
}
