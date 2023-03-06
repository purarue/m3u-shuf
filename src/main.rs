use std::fmt;
use std::fs;
use std::io;
use std::str::FromStr;

use rand::seq::SliceRandom;
use rand::thread_rng;

use anyhow::{bail, Result, Context};
use clap::Parser;

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = "CLI tool to shuffle a m3u file. If no file given, reads from STDIN"
)]
struct Cli {
    /// m3u file to shuffle
    file: Option<String>,

    // output file to write to. If not given, writes to STDOUT
    #[clap(short, long)]
    output: Option<String>,
}

// note: these contain newlines
struct Track {
    extinf: Option<String>,
    path: String,
}

impl fmt::Display for Track {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(extinf) = &self.extinf {
            writeln!(f, "{}", extinf)?;
        }
        write!(f, "{}", self.path)
    }
}

struct M3U {
    pub tracks: Vec<Track>,
}

const EXTM3U: &str = "#EXTM3U";
const EXTINF: &str = "#EXTINF";

impl FromStr for M3U {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        // make sure the first line is the header
        if !lines.next().unwrap_or_default().starts_with(EXTM3U) {
            bail!("Missing #EXTM3U header");
        }
        let mut tracks = Vec::new();
        let mut extinf = None;
        for line in lines {
            if line.trim().is_empty() {
                continue;
            } else if line.starts_with(EXTINF) {
                extinf = Some(line.trim_end().to_string());
            } else {
                tracks.push(Track {
                    extinf,
                    path: line.trim_end().to_string(),
                });
                extinf = None;
            }
        }
        Ok(M3U { tracks })
    }
}

impl fmt::Display for M3U {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", EXTM3U)?;
        for track in &self.tracks {
            writeln!(f, "{}", track)?;
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let args = Cli::parse();

    // if args.file is None, read from STDIN
    let buffer = match args.file {
        Some(ref file) => fs::read_to_string(file).context(format!("Unable to read from file '{}'", file))?,
        None => io::read_to_string(io::stdin()).expect("Unable to read from STDIN")
    };

    // parse, shuffle
    let m3u = buffer.parse::<M3U>().context("Unable to parse into m3u format")?;
    let mut tracks = m3u.tracks;
    tracks.shuffle(&mut thread_rng());

    // serialize
    let out = M3U { tracks }.to_string();

    // write to STDOUT
    match args.output {
        Some(ref file) => fs::write(file, out).context(format!("Unable to write to file '{}'", file))?,
        None => print!("{}", out),
    }

    Ok(())
}
