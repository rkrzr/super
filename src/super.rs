const DOCUMENTATION: &str = "NAME
        super - manage all of your git repos in one super repository

SYNOPSIS
        super init - Initialize a new super repo for the first time. This is just a convenience wrapper
             around 'git init'.

        super add - Add a new repo to the super repo. This is just a convenience wrapper
            around 'git submodule add'.

        super pull - Update all repos in the super repo.

        super foreach <command> - [TODO] Run a regular shell command for each repo in parallel

DESCRIPTION
        Super is a tool that enables you to manage all of your git repos in one centralized repository.
        It is based on the idea of a super repository, which is a collection of git repos that can be
        managed together. Typically, the repos belong together somehow, but this is not a hard requirement.

        Super makes use of git submodules. It discovers all submodules in .gitmodules and pulls in their
        latest code when running \"super pull\". Super is thus a wrapper around existing git functionality
        with the goal to make using submodules more convenient by adding an intuitive CLI and a colorful
        terminal UI.

AUTHOR
        Written by Robert Kreuzer.

REPORTING BUGS
        https://github.com/rkrzr/super/issues

COPYRIGHT
        Copyright © 2023 Robert Kreuzer.  License BSD-3-Clause: The 3-Clause BSD License <https://opensource.org/license/bsd-3-clause/>.
        This is free software: you are free to change and redistribute it.  There is NO WARRANTY, to the extent permitted by law.";

use git2::Repository;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
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

// The main function. It parses CLI args and calls the right handler function.
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        // Print the docs with usage instructions
        println!("{}", DOCUMENTATION);
        println!("Git repos: {:?}", get_git_repos());
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
        } else if args[1] == "foreach" {
            // Note: all arguments after "super foreach" are interpreted as the command to
            // run in each submodule.
            if args.len() < 3 {
                println!("Usage: super foreach <command>");
            } else {
                match command_foreach(&args[2..]) {
                    Ok(_) => (),
                    Err(error) => println!("Error running command: {:?}", error),
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

// Run the given command for each submodule in parallel
fn command_foreach(command: &[String]) -> Result<(), git2::Error> {
    // TODO: Deduplicate the next two lines.
    let repo: Repository = Repository::open(".")?;
    let current_dir: std::path::PathBuf =
        env::current_dir().expect("Failed to get current directory");

    // Run the given command as a subprocess for each submodule
    let mut threads = vec![];

    for submodule in repo.submodules()? {
        let name = submodule.name().unwrap_or("").to_string();
        let repo_dir = current_dir.join(name.clone());

        let cmd: Vec<String> = command.to_vec();
        let handle = thread::spawn(move || run_command(&repo_dir, cmd));
        threads.push(handle);
    }

    // Wait for all threads to finish
    for handle in threads {
        handle.join().unwrap();
    }

    Ok(())
}

// Run the given command as a subprocess (but not in a sub-shell).
// The output of the command is printed to stdout.
fn run_command(repo_path: &PathBuf, cmd: Vec<String>) -> () {
    let mut command = Command::new(cmd[0].clone());

    // Add all arguments to the command
    if cmd.len() > 1 {
        command.args(&cmd[1..]);
    }

    let output: Output = command
        .current_dir(repo_path)
        .output()
        .expect("failed to execute process");

    if !output.status.success() {
        print!(
            "Failed to run the command in the submodule. Error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    } else {
        println!("{}", String::from_utf8_lossy(&output.stdout));
    }
}

// Pull all submodules in the given repo in parallel
fn pull_in_parallel(current_dir: &PathBuf) -> Result<(), git2::Error> {
    let mut threads = vec![];

    // Vector of (repo_path, repo_name, branch) tuples
    let mut repos: Vec<(PathBuf, String, String)> = vec![];

    match Repository::open(".") {
        // Case 1: The directory that 'super' was called in, is a git repo itself
        Ok(repo) => {
            match repo.submodules() {
                Ok(submodules) => {
                    for submodule in submodules {
                        let name = submodule.name().unwrap_or("").to_string();
                        let repo_dir = current_dir.join(name.clone());

                        // submodules can specify a default branch in .gitmodules. We pull that branch by
                        // default, and otherwise we pull "master"
                        let branch = submodule.branch().unwrap_or("master").to_string();

                        repos.push((repo_dir, name, branch))
                    }
                }
                Err(error) => {
                    println!("Failed to get submodules: {}", error)
                }
            }
        }
        // Case 2: The directory that 'super' was called in, is *not* a git repo itself
        Err(_error) => {
            for repo_name in get_git_repos() {
                let repo_dir = current_dir.join(&repo_name);

                // We want to pull the currently checked out branch
                let branch = get_current_branch(&repo_dir);

                repos.push((repo_dir, repo_name, branch))
            }
        }
    }

    for (repo_dir, repo_name, branch) in repos.into_iter() {
        let handle = thread::spawn(move || {
            pull_single_repo(&repo_dir, &repo_name, &branch);
        });
        threads.push(handle);
    }

    // Wait for all threads to finish
    for handle in threads {
        handle.join().unwrap();
    }

    Ok(())
}

// Fetch the latest commits for the given branch, and do a fast-forward merge
// if, and only if, the repo is on the given branch and has no uncommitted changes.
fn pull_single_repo(repo_dir: &PathBuf, name: &str, branch: &str) -> () {
    let hash_before = get_head_sha(repo_dir);
    // Fetch the latest commits
    git_fetch(repo_dir, branch);

    // Get the currently checked out branch
    let branch_name = get_current_branch(repo_dir);

    if branch_name != branch {
        print_status_line(name, &PullStatus::Unchanged, "not on tracked branch");
        return;
    }

    forward_branch(repo_dir, branch);

    let hash_after = get_head_sha(repo_dir);
    let short_hash_before = get_short_hash(repo_dir, &hash_before);
    let short_hash_after = get_short_hash(repo_dir, &hash_after);

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
    let current_dir: std::path::PathBuf =
        env::current_dir().expect("Failed to get current directory");

    pull_in_parallel(&current_dir)
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
fn get_short_hash(repo_dir: &PathBuf, committish: &String) -> String {
    let output: Output = Command::new("git")
        .arg("rev-parse")
        .arg("--short")
        .arg(committish)
        .current_dir(repo_dir)
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

/// Get the user's custom commands from ~/.config/super/commands
fn get_commands() -> Vec<String> {
    if let Some(home_dir) = dirs::home_dir() {
        // Get all executable files
        let directory_path: PathBuf = PathBuf::from(".config/super/commands");
        let combined_path = home_dir.join(&directory_path);

        // Collect all commands
        let mut commands: Vec<String> = Vec::new();

        match fs::read_dir(&combined_path) {
            Ok(entries) => {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let file_name = entry.file_name();

                        if let Ok(metadata) = entry.metadata() {
                            if metadata.is_dir() {
                                println!("Skipping {:?} because it is a directory.", entry.path())
                            } else {
                                // We only want to add executable files to the list of commands
                                if metadata.permissions().mode() & 0o111 != 0 {
                                    commands.push(file_name.to_string_lossy().to_string());
                                } else {
                                    println!(
                                        "Skipping {:?} because it is not executable.",
                                        entry.path()
                                    )
                                }
                            }
                        } else {
                            println!("Skipping {:?} because of invalid metadata.", entry.path())
                        }
                    }
                }

                return commands;
            }
            Err(error) => {
                eprintln!("Error reading {:?}: {:?}", &combined_path, error);
                return Vec::new();
            }
        }
    } else {
        eprintln!("Unable to determine the home directory");
        return Vec::new();
    }
}

// This function discovers all git repos in the current directory
// that super is invoked in.
fn get_git_repos() -> Vec<String> {
    // We use 'find' to discover all repos with a .git directory
    let output: Output = Command::new("find")
        .arg(".")
        // Optimization: Limit search depth to 2 levels
        .arg("-maxdepth")
        .arg("2")
        .arg("-name")
        .arg(".git")
        .output()
        .expect("failed to execute the find process");

    if !output.status.success() {
        print!(
            "Failed to discover all git repos. Error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Vec::new();
    } else {
        let lines = String::from_utf8_lossy(&output.stdout)
            .trim()
            .lines()
            // Drop the "/.git" at the end of the path
            .map(|x| x.to_string().replace("/.git", ""))
            .collect();
        return lines;
    }
}
