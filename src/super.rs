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

use git2::Repository;
use std::env;
use std::path::PathBuf;
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
                match command_pull() {
                    Ok(_) => (),
                    Err(error) => println!("Error pulling your repos: {:?}", error),
                }
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
fn command_pull() -> Result<(), git2::Error> {
    let repo: Repository = Repository::open(".")?;
    let current_dir: std::path::PathBuf =
        env::current_dir().expect("Failed to get current directory");

    for submodule in repo.submodules()? {
        let name: &str = submodule.name().unwrap_or("");
        // Fetch the branch specified in .gitmodules
        let branch = submodule.branch();
        let repo_dir = current_dir.join(name);

        if let Some(branch) = branch {
            git_fetch(&repo_dir, branch);
        } else {
            // println!(
            //     "The {:?} repo has no branch specified in .gitmodules. Defaulting to 'master'.",
            //     submodule.name()
            // );
            // TODO: Print output for each repo here
            git_fetch(&repo_dir, "master");

            // Fast-forward the branch to the latest commit
            let hash_before = get_head_sha(&repo_dir);
            forward_branch(&repo_dir, "master");
            let hash_after = get_head_sha(&repo_dir);
            print_status_line(name, &hash_before, &hash_after)
        }
    }

    Ok(())
}

/// Fetch the branch that is specified in .gitmodules.
fn git_fetch(repo_dir: &PathBuf, branch: &str) {
    let output: Output = Command::new("git")
        .arg("fetch")
        // TODO: Don't specify the remote here? Git, by default, will use the
        // origin remote, unless there's an upstream branch configured for the current
        // branch
        .arg("origin")
        .arg(branch)
        .current_dir(repo_dir)
        .output()
        .expect("failed to execute process");

    if !output.status.success() {
        print!(
            "Failed to fetch the repo. Error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

/// Fast-forward the given branch, in the given repo.
fn forward_branch(repo_dir: &PathBuf, branch: &str) {
    let output: Output = Command::new("git")
        .arg("merge")
        .arg("--ff-only")
        // TODO: Don't hardcode the remote here
        .arg("origin")
        .arg(branch)
        .current_dir(repo_dir)
        .output()
        .expect("failed to execute process");

    if !output.status.success() {
        print!(
            "Failed to fast-forward the repo. Error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

/// Print the status of the given repo
fn print_status_line(repo: &str, hash_before: &String, hash_after: &String) {
    let short_hash_before = get_short_hash(hash_before);
    let short_hash_after = get_short_hash(hash_after);


    // neon pink (\x1b[38;5;198;1m), bright cyan(\x1b[1;36), white (\x1b[1;37m)
    println!(
        "\x1b[38;5;198;1m{repo:18} \x1b[1;36m     updated \x1b[1;37m      ({short_hash_before}) -> ({short_hash_after})\x1b[0m"
    )
}

/// Return the commit hash that HEAD points to.
fn get_head_sha(repo_dir: &PathBuf) -> String {
    return resolve_ref(repo_dir, "HEAD".to_string());
}

/// Return the hash of the commit (or tag) that the ref points to.
fn resolve_ref(repo_dir: &PathBuf, committish: String) -> String {
    let output: Output = Command::new("git")
        .arg("log")
        .arg("-1")
        .arg("--format=format:%H")
        .arg(committish)
        .current_dir(repo_dir)
        .output()
        .expect("failed to execute process");

    if !output.status.success() {
        print!(
            "Failed to resolve the given reference. Error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    return output.stdout.escape_ascii().to_string();
}

/// Return a 7 character long hash for a given commit.
fn get_short_hash(committish: &String) -> String {
    let output: Output = Command::new("git")
        .arg("rev-parse")
        .arg("--short")
        .arg(committish)
        // .current_dir(repo_dir)
        .output()
        .expect("failed to execute process");

    if !output.status.success() {
        print!(
            "Failed to get a short hash for the given commit. Error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    return stdout.trim().to_string()
}
