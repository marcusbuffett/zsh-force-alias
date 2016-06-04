extern crate hyper;

use hyper::*;
use std::io::Read;
use std::env;
mod util;

static BASE_URL: &'static str = "http://localhost:5571/";
// TODO: work with vi mode in zsh

fn main() {
    let client = Client::new();
    let mut args: Vec<String> = env::args().collect();
    // If called with `--init`, initialize the server with
    // aliases
    if args.get(1) == Some(&"--init".to_string()) {
        send_aliases(&client);
        std::process::exit(0);
    }
    // Remove the file name from the args
    args.remove(0);
    // vec!["git", "status"] -> "git status"
    let command = args.join(" ");
    send_command(client, &command);
}

// Sends a command to the server
fn send_command(client: Client, command: &String) {
    let url: String = BASE_URL.to_string() + "commands";
    let res_or_err = client.post(&url).body(command).send();
    // If server isn't running
    if res_or_err.is_err() {
        return;
    }
    let mut s = String::new();
    let mut res = res_or_err.unwrap();
    res.read_to_string(&mut s).unwrap();
    // Exit with exit code of 0 if successful, and 1 if
    // unsuccessful (aka the server didn't like the command)
    match res.status {
        hyper::Ok => {
            std::process::exit(0);
        }
        _ => {
            println!("{}", s);
            std::process::exit(1);
        }
    }
}

// Send aliases to the server to be parsed and saved
fn send_aliases(client: &Client) {
    let url: String = BASE_URL.to_string() + "aliases";
    let command = "alias -L".to_string();
    // Have to manually load ~/.zshrc and /etc/zshrc because
    // zsh doesn't respect the --login and -c flags together
    let full_command = format!(
    "export NO_CHECK=1; \
    [ -f /etc/zshrc ] && . /etc/zshrc; \
    [ -f ~/.zshrc ] && . ~/.zshrc; \
    {}", command);
    // Spawn new process with command
    let output = std::process::Command::new("zsh")
                     .arg("-c")
                     .arg(full_command)
                     .output()
                     .unwrap();
    let stdout = std::str::from_utf8(&*output.stdout).unwrap();
    // Post aliases to server
    let mut res = client.post(&url).body(stdout).send().unwrap();
    match res.status {
        hyper::Ok => {
            std::process::exit(0);
        }
        _ => {
            let mut s = String::new();
            res.read_to_string(&mut s).unwrap();
            std::process::exit(1);
        }
    }
}
