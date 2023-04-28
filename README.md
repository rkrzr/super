# super

One super repo to rule them all.

```bash
# Dev shell
nix develop

# Build
nix build
```

## How to use `super` to organize your git repos

1. Install using `super` locally using one of these methods: ...
2. Create a new directory that will contain all your git repos, that you want to manage with `super`. Let's call this directory `repos` (create it with `mkdir repos`).
3. Enter the new directory with `cd repos`
4. In this directory run `super init` to initialize it as a new git repo
5. Add all of your git repos with `super add <pathspec>`

## cargo

```bash
cargo build [--release]
cargo run [--release]
cargo test [--release]

# Run quietly with one argument supplied
cargo run --quiet add
```

## git

Useful for local development within the `super` repo:

- you can exclude files locally in `.git/info/exclude`. The syntax is the same as in `.gitignore`