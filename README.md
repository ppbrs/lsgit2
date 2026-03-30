# lsgit2

An application that recursively searches for git repositories and lists some useful information about them.
This project is inspired by `lsgit` but is going to have more functions.

# Usage

lsgit2 [OPTIONS] [REGEX_PATTERN]

Arguments:
  [REGEX_PATTERN]  The regexp pattern to filter the paths of git repositories or their git-status strings, which contain the name of the checkout branch


Options:
  -d, --depth <DEPTH>          Max number of subdirectories to seach in [default: 1]
  -s, --start-dir <START_DIR>  The directory where to start the search [default: .]
  -h, --help                   Print help
  -V, --version                Print version

# Instructions for maintainers

Use the following command to install a locally built executable to `~/.cargo/bin`:
```
cargo install --path .
```