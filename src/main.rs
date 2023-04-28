/*
This file contains the implementation of "super", a tool to manage all of your
git repos in one super repo.

It was created by Robert Kreuzer in 2023.

# Usage

super init - Initialize a new super repo for the first time. This is just a convenience wrapper
             around 'git init'.

super add - Add a new repo to the super repo. This is just a convenience wrapper
            around 'git submodule add'.

super pull - Update all repos in the super repo.
*/

use std::env;
use std::process::Command;
use std::process::Output;

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
        } else if args[1] == "init" {
            if args.len() != 2 {
                println!("Usage: super init")
            } else {
                command_init()
            }
        } else if args[1] == "pull" {
            if args.len() != 2 {
                println!("Usage: super pull")
            } else {
                command_pull()
            }
        } else {
            println!("We only support the 'super add' command right now.");
        }
    }
}

/// Initialize the super repo for the first time
///
/// You have to call this in the directory that you want to initialize
fn command_init() {
    let output = Command::new("git")
        .arg("init")
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        println!("The super repo was initialized successfully.");
        println!("You can now add your repos with 'super add <pathspec>")
    } else {
        print!(
            "Failed to initialize the super repo. Error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

/// Add a new repo to the super repo
///
/// This will add the repo as a submodule and will also initialize it
fn command_add(repo_path: &String) {
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

/// Pull the latest code for all submodules in the super repo
///
/// This will add the repo as a submodule and will also initialize it
fn command_pull() {
    let output = Command::new("git")
        .arg("submodule")
        .arg("update")
        .arg("--init")
        .arg("--recursive")
        // Updates are printed to stderr by git
        .arg("--progress")
        .arg("--jobs")
        .arg("4")
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        println!("All repos were updated successfully.");
    } else {
        print!(
            "Failed to update the repos. Error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

/// Fetch the branch that is specified in .gitmodules.
/// If no branch is specified we fetch all of them.
fn git_fetch(branch: &String) {
    let output: Output = Command::new("git")
        .arg("fetch")
        // TODO: Don't hard-code origin here, but discover it per repo
        .arg("origin")
        .arg(branch)
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        println!("All repos were fetched successfully.");
    } else {
        print!(
            "Failed to fetch the repos. Error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
