# super

`super` is a small CLI utility to manage multiple git repositories that reside
in the same directory. The typical use case is that you have one *super repo*
which contains a number of other git repos as submodules, listed in
`.gitmodules`. It is written in Rust.

You can then run `super pull` to pull all of the submodules in parallel. `super`
will only pull the branch that has been specified in `.gitmodules`, otherwise it
will fall back to `master`. If a repo is on a different branch, or if it has
uncommitted changes, then `super` will do nothing and skip the repo.

`super pull` prints a nice summary of the changes for each repo:

![terminal output of super pull](https://github.com/rkrzr/super/assets/82817/d75a810a-c03e-4c25-8b93-86678c2ab0e2)


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