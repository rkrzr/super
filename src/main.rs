use std::env;

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

fn command_add(repo_path: &String) {
    println!("Super add! (to be implemented) {}", repo_path)
}
