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

super foreach <command> - [TODO] Run a regular shell command for each repo in parallel
*/

use git2::Repository;
use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::thread;

/// The status of the pull operation
#[derive(PartialEq)]
enum PullStatus {
    Unchanged,
    Updated,
    UpToDate,
}

impl PullStatus {
    fn to_str(&self) -> &str {
        match *self {
            PullStatus::Unchanged => "unchanged",
            PullStatus::Updated => "updated",
            PullStatus::UpToDate => "up to date",
        }
    }
}

impl std::fmt::Display for PullStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

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

fn pull_in_parallel(current_dir: &PathBuf, repo: &Repository) -> Result<(), git2::Error> {
    let mut threads = vec![];
    for submodule in repo.submodules()? {
        let name = submodule.name().unwrap_or("").to_string();
        let repo_dir = current_dir.join(name.clone());

        // submodules can specify a default branch in .gitmodules. We pull that branch by
        // default, and otherwise we pull "master"
        let branch = submodule.branch().unwrap_or("master").to_string();

        let handle = thread::spawn(move || {
            pull_single_repo(&repo_dir, &name, &branch);
        });
        threads.push(handle);

        // Wait for the thread to finish
    }
    for handle in threads {
        handle.join().unwrap();
    }

    Ok(())
}

// Fetch the latest commits for the given branch, and do a fast-forward merge
// if, and only if, the repo is on the given branch and has no uncommitted changes.
fn pull_single_repo(repo_dir: &PathBuf, name: &str, branch: &str) -> () {
    let hash_before = get_head_sha(&repo_dir);
    // Fetch the latest commits
    git_fetch(&repo_dir, branch);

    // Get the currently checked out branch
    let branch_name = get_current_branch(&repo_dir);

    if branch_name != branch {
        // println!("Current branch: {}", branch_name);
        // println!("The repo {} is not on branch {}. Skipping.", name, branch);
        print_status_line(name, &PullStatus::Unchanged, "not on tracked branch");
        return;
    }

    forward_branch(&repo_dir, branch);

    let hash_after = get_head_sha(&repo_dir);
    let short_hash_before = get_short_hash(&hash_before);
    let short_hash_after = get_short_hash(&hash_after);

    if hash_before == hash_after {
        let status = PullStatus::UpToDate;
        let remark: String = format!("{branch}({short_hash_before})");
        print_status_line(name, &status, &remark);
    } else {
        let status = PullStatus::Updated;
        let remark: String =
            format!("{branch}({short_hash_before}) -> {branch}({short_hash_after})");
        print_status_line(name, &status, &remark);
    };
}

/// Get the current branch of the repo
fn get_current_branch(repo_dir: &PathBuf) -> String {
    let output: Output = Command::new("git")
        .arg("branch")
        .arg("--show-current")
        .current_dir(repo_dir)
        .output()
        .expect("failed to execute process");

    if !output.status.success() {
        print!(
            "Failed to fetch the repo. Error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = output.stdout.to_ascii_lowercase();
    return String::from_utf8_lossy(&stdout).trim().to_string();
}

/// Pull the latest code for all submodules in the super repo
fn command_pull() -> Result<(), git2::Error> {
    let repo: Repository = Repository::open(".")?;
    let current_dir: std::path::PathBuf =
        env::current_dir().expect("Failed to get current directory");

    pull_in_parallel(&current_dir, &repo)
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
fn print_status_line(repo: &str, status: &PullStatus, remark: &str) {
    // Note: We have to convert the pull status to a string first, because we want to align the string,
    // and alignment is not implemented for the Debug trait.
    let status_str = status.to_str();

    // neon pink (\x1b[38;5;198;1m), bright cyan(\x1b[1;36), white (\x1b[1;37m)
    println!("\x1b[38;5;198;1m{repo:16} \x1b[1;36m{status_str:10} \x1b[1;37m   {remark}\x1b[0m")
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
    return stdout.trim().to_string();
}
