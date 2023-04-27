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
    println!("Super add! (to be implemented) {}", repo_path);

    // Run a git subprocess
    let output = Command::new("git")
        .arg("status")
        .output()
        .expect("failed to execute process");

    println!("{}", String::from_utf8_lossy(&output.stdout));
}
