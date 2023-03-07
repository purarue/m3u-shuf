use std::fmt;
use std::fs;
use std::io;
use std::str::FromStr;

use rand::seq::SliceRandom;
use rand::thread_rng;

use anyhow::{bail, Context, Result};
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

    /// output file to write to
    #[clap(short, long)]
    output: Option<String>,
}

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

impl M3U {
    fn shuffle(&mut self) {
        self.tracks.shuffle(&mut thread_rng());
    }
}

trait TrimNewline {
    fn trim_newline(&mut self) -> Self;
}

impl TrimNewline for String {
    fn trim_newline(&mut self) -> String {
        let len = self.trim_end_matches(&['\r', '\n'][..]).len();
        self.truncate(len);
        self.to_string()
    }
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
            let ln = line.to_string().trim_newline();
            if ln.trim().is_empty() {
                continue;
            } else if ln.starts_with(EXTINF) {
                extinf = Some(ln);
            } else {
                tracks.push(Track { extinf, path: ln });
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
        Some(ref file) => {
            fs::read_to_string(file).context(format!("Unable to read from file '{}'", file))?
        }
        None => io::read_to_string(io::stdin()).context("Unable to read from STDIN")?,
    };

    // parse, shuffle
    let mut m3u: M3U = buffer.parse().context("Unable to parse into m3u format")?;
    m3u.shuffle();

    // write to file or STDOUT
    match args.output {
        Some(ref file) => fs::write(file, m3u.to_string())
            .context(format!("Unable to write to file '{}'", file))?,
        None => print!("{}", m3u),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str;

    #[test]
    fn test_basic() {
        let m3u: M3U = M3U::from_str(
            r#"#EXTM3U
#EXTINF:0,Artist1 - Title1
path/to/file1.mp3
#EXTINF:0,Artist2 - Title2
path/to/file2.mp3
"#,
        )
        .unwrap();
        assert_eq!(m3u.tracks.len(), 2);
        assert_eq!(m3u.tracks[0].path, "path/to/file1.mp3");
        assert_eq!(m3u.tracks[1].path, "path/to/file2.mp3");
        assert_eq!(
            m3u.tracks[0].extinf,
            Some("#EXTINF:0,Artist1 - Title1".to_string())
        );
        assert_eq!(
            m3u.tracks[1].extinf,
            Some("#EXTINF:0,Artist2 - Title2".to_string())
        );
    }

    #[test]
    fn test_windows_newline() {
        let win_newline: String = str::from_utf8(&[b'\r', b'\n']).unwrap().to_string();
        let buf = format!(
            "{}{}",
            r#"#EXTM3U
#EXTINF:0,Artist1 - Title1
path/to/file1.mp3"#,
            win_newline
        );
        assert_eq!(buf[buf.len() - 2..], win_newline);
        let m3u: M3U = M3U::from_str(&buf).unwrap();
        assert_eq!(m3u.tracks.len(), 1);
        assert_eq!(m3u.tracks[0].path, "path/to/file1.mp3");
        assert_eq!(
            m3u.tracks[0].extinf,
            Some("#EXTINF:0,Artist1 - Title1".to_string())
        );

        // reserialize to test if windows newline was removed
        let out = M3U { tracks: m3u.tracks }.to_string();
        let mut ser = buf.clone().trim_newline();
        ser.push_str("\n");
        assert_eq!(out, ser);
    }
}
