# m3u-shuf

This supports a subset of the extended m3u format - requires the `#EXTM3U` header and keeps `#EXTINF` attached to each entry while shuffing.

```
CLI tool to shuffle a m3u file

Usage: m3u-shuf [OPTIONS] [FILE]

Arguments:
  [FILE]  m3u file to shuffle

Options:
  -o, --output <OUTPUT>  output file to write to
  -h, --help             Print help (see more with '--help')
  -V, --version          Print version
```

## Install

Install [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html), then:

```
cargo install --git https://github.com/seanbreckenridge/m3u-shuf
```

## Example Usage

`m3u-shuf playlist.m3u -o shuffled.m3u`

If no output file is specified, it will write to STDOUT; can be piped to another command, or redirected to a file:

`m3u-shuf playlist.m3u > shuffled.m3u`

`m3u-shuf | tee shuffled.m3u`

Shuffling in place:

`m3u-shuf playlist.m3u -o playlist.m3u`

If no input file is specified, it will read from STDIN (e.g. with [`plainplay`](https://github.com/seanbreckenridge/plaintext-playlist)):

`plainplay m3u rock | m3u-shuf | tee rock.m3u`
