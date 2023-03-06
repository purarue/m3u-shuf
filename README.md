# m3u-shuf

This supports a subset of the extended m3u format - requires the `#EXTM3U` header and keeps `#EXTINF` attached to each entry while shuffing.

```
CLI tool to shuffle a m3u file

Usage: m3u-shuf [OPTIONS] [FILE]

Arguments:
  [FILE]  m3u file to shuffle

Options:
  -o, --output <OUTPUT>  
  -h, --help             Print help (see more with '--help')
  -V, --version          Print version
```

### Install

Install [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html), then:

```
cargo install --git https://github.com/seanbreckenridge/m3u-shuf
```

#### Example Usage

`m3u-shuf playlist.m3u > shuffled.m3u`

`m3u-shuf playlist.m3u -o shuffled.m3u`

This supports shuffling in place, e.g.:

`m3u-shuf playlist.m3u -o playlist.m3u`

... and reading from STDIN:

`cat playlist.m3u | m3u-shuf | tee shuffled.m3u`
