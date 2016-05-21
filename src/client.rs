extern crate hyper;

use hyper::*;
use std::io::Read;
use std::env;
mod util;

static BASE_URL: &'static str = "http://localhost:5671/";

fn main() {
    let client = Client::new();
    let mut args: Vec<String> = env::args().collect();
    // TODO: expect that server returns 200 for list_aliases or smth
    if args.get(1) == Some(&"--init".to_string()) {
        send_aliases(&client);
        std::process::exit(0);
    }
    args.remove(0);
    let command = args.join(" ");
    send_command(client, &command);
}

fn send_command(client: Client, command: &String) {
    let url: String = BASE_URL.to_string() + "commands";
    let mut res = client.post(&url).body(command).send().unwrap();
    let mut s = String::new();
    res.read_to_string(&mut s).unwrap();
    match res.status {
        hyper::Ok => {
            println!("{}", s);
            std::process::exit(0);
        }
        _ => {
            println!("I'm sorry Dave, I can't let you do that.");
            println!("Maybe you should use this command instead:");
            println!("{}", s);
            std::process::exit(1);
        }
    }
}

fn send_aliases(client: &Client) {
    let url: String = BASE_URL.to_string() + "aliases";
    println!("INITING!");
    let mut command = String::new();
    command = "alias -L".to_string();
    let full_command = format!(
    "export DISABLE_CLIENT=1; \
    [ -f /etc/zshrc ] && . /etc/zshrc; \
    [ -f ~/.zshrc ] && . ~/.zshrc; \
    {}", command);
    println!("{:?}", full_command);
    let output = std::process::Command::new("zsh")
                     .arg("-c")
                     // TODO: make this less hacky
                     .arg(full_command)
                     .output()
                     .unwrap();
    println!("{:?}", output);
    let stdout = std::str::from_utf8(&*output.stdout).unwrap();
    let mut res = client.post(&url).body(stdout).send().unwrap();
    match res.status {
        hyper::Ok => {
            print!("Successful upload!");
            std::process::exit(0);
        }
        _ => {
            let mut s = String::new();
            res.read_to_string(&mut s).unwrap();
            println!("{}", s);
            std::process::exit(1);
        }
    }
}
