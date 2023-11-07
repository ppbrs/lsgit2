# lsgit2

An application that recursively searches for git repositories and lists some useful information about them.
This project is inspired by `lsgit` but is going to have more functions.

# Usage

`lsgit2 [start-directory] [regex-pattern]`
`start-directory` is where to start searching for git repositories.
`regex-pattern` is a pattern to filter the paths of git repositories or their status strings usually containing the name of the checkout branch.

# Instructions for maintainers

Use the following command to install a locally built executable to `~/.cargo/bin`:
```
cargo install --path .
```