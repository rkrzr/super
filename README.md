# super

`super` is a small CLI utility to manage multiple git repositories that reside
in the same directory. The typical use case is that you have one *super repo*
which contains a number of other git repos as submodules, listed in
`.gitmodules`. It is written in Rust.

You can run `super pull` to pull all of the submodules in parallel. `super`
will only pull the branch that has been specified in `.gitmodules`, otherwise it
will fall back to `master`. If a repo is on a different branch, or if it has
uncommitted changes, then `super` will do nothing and skip the repo.

`super pull` prints a nice summary of the changes for each repo:

![terminal output of super pull](https://github.com/rkrzr/super/assets/82817/6a99b27a-a52a-4fb4-b7ef-1669c3dc544e)


## How to use `super` to organize your git repos

1. Install `super` locally using one of these methods: ...
2. Create a new directory that will contain all your git repos, that you want to
   manage with `super`. Let's call this directory `repos` (create it with `mkdir
   repos`).
3. Enter the new directory with `cd repos`
4. In this directory run `super init` to initialize it as a new git repo
5. Add all of your git repos with `super add <pathspec>`

## Development setup

This repo makes use of Nix and direnv. Run `direnv allow` to get a shell that
has the right Rust version (specified in `rust-toolchain`) on its PATH. All Rust
dependencies are specified in `Cargo.toml`. All non-Rust dependencies are
specified in `shell.nix` (e.g. `cmake` and `clang`).

###  cargo

Useful cargo commands:

```bash
cargo fmt
cargo build [--release]
cargo run [--release]
cargo test [--release]

# Run quietly with one argument supplied
cargo run --quiet add

# Install to $HOME/.cargo/bin
cargo install --path .
```

### Updating dependencies

Run `cargo update` to update all dependencies tracked in `Cargo.lock` to the
newest version. Then commit and push the changes.

The Rust version is pinned in `rust-toolchain`.