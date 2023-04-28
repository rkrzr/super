/*
This file contains the implementation of "super", a tool to manage all of your
git repos in one super repo.

It was created by Robert Kreuzer in 2023.

# Usage

super add - Add a new repo to the super repo. This is just a convenience wrapper
            around 'git submodule add'.
*/

use std::env;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();

    // println!("{:?}", args);
    if args.len() < 2 {
        println!("Usage: super <command>")
    } else {
        if args[1] == "add" {
            if args.len() != 3 {
                println!("Usage: super add <repo_path>")
            } else {
                let repo_path = &args[2];
                command_add(repo_path)
            }
        } else {
            println!("We only support the 'super add' command right now.")
        }
    }
}

/// Add a new repo to the super repo
///
/// This will add the repo as a submodule and will also initialize it
fn command_add(repo_path: &String) {
    // println!("Super add! (to be implemented) {}", repo_path);

    // Run a git subprocess
    let output = Command::new("git")
        .arg("submodule")
        .arg("add")
        // TODO: We might want to pass along all optional arguments here
        .arg(repo_path)
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        println!("The submodule {} was added successfully.", repo_path);
        println!("You probably will want to commit this (along with .gitmodules, if this is the first submodule.")
    } else {
        print!(
            "Failed to add the submodule. Error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
