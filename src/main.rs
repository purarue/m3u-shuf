use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

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

const EXTM3U: &str = "#EXTM3U";
const EXTINF: &str = "#EXTINF";

impl TryFrom<Box<dyn BufRead>> for M3U {
    type Error = anyhow::Error;

    fn try_from(buf: Box<dyn BufRead>) -> Result<M3U, anyhow::Error> {
        let mut lines = buf.lines();
        // make sure the first line is the header
        if !lines
            .next()
            .context("cannot read empty input")?
            .context("cannot read line")?
            .starts_with(EXTM3U)
        {
            bail!("Missing #EXTM3U header");
        }
        let mut tracks = Vec::new();
        let mut extinf = None;
        for line in lines {
            // bufread already trims newline properly
            let ln = line.context("cannot read line")?.to_string();
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

    let stdin = io::stdin();
    let stdout = io::stdout();

    let mut m3u: M3U;
    // scope to drop reader after parsing
    {
        // if args.file is None, read from STDIN
        let reader: Box<dyn BufRead> = match args.file {
            Some(ref file) => Box::new(BufReader::new(
                File::open(file).context(format!("Unable to open file to read from '{}'", file))?,
            )),
            None => Box::new(stdin.lock()),
        };

        // parse
        m3u = reader.try_into().context("Unable to parse m3u file")?;
    }
    // shuffle
    m3u.shuffle();

    // scope to drop writer after writing, before program exits
    {
        // write to file or STDOUT
        let mut out: Box<dyn Write> = match args.output {
            Some(ref file) => File::create(file)
                .map(|f| Box::new(f) as Box<dyn Write>)
                .context(format!("Unable to open file to write to '{}'", file))?,
            None => Box::new(stdout.lock()),
        };

        write!(out, "{}", m3u).context("Unable to write to output file")?;
        out.flush()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str;

    #[test]
    fn test_basic() {
        let file = File::open("testdata/basic.m3u").unwrap();
        let buf = Box::new(BufReader::new(file)) as Box<dyn BufRead>;
        let m3u: M3U = M3U::try_from(buf).unwrap();
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
        // create the testdata file if it doesn't exist
        let win_newline: String = str::from_utf8(&[b'\r', b'\n']).unwrap().to_string();
        let buf = format!(
            "{}{}",
            r#"#EXTM3U
#EXTINF:0,Artist1 - Title1
path/to/file1.mp3"#,
            win_newline
        );
        assert_eq!(buf[buf.len() - 2..], win_newline);
        let filename = "testdata/windows_newline.m3u";
        let testdata = std::path::Path::new(filename);
        if !testdata.exists() {
            // write to file
            let mut file = File::create(filename).unwrap();
            file.write_all(buf.as_bytes()).unwrap();
        }

        // read from file
        let file = File::open(filename).unwrap();
        let bufreader = Box::new(BufReader::new(file)) as Box<dyn BufRead>;
        let m3u: M3U = M3U::try_from(bufreader).unwrap();
        assert_eq!(m3u.tracks.len(), 1);
        assert_eq!(m3u.tracks[0].path, "path/to/file1.mp3");
        assert_eq!(
            m3u.tracks[0].extinf,
            Some("#EXTINF:0,Artist1 - Title1".to_string())
        );

        // reserialize to test if windows newline was removed
        let out = M3U { tracks: m3u.tracks }.to_string();
        let mut ser = buf.clone().trim_end_matches(&['\r', '\n'][..]).to_string();
        ser.push_str("\n");
        assert_eq!(out, ser);
    }
}
