#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate hyper;
extern crate argparse;
extern crate serde_json;

use hyper::{Result, Client};
use std::io::Read;
use argparse::*;

mod util;
mod request_types;

use request_types::{PostDeclarations, PostCommand};

static BASE_URL: &'static str = "http://localhost:5571/";
// TODO: work with vi mode in zsh

fn main() {
    let client = Client::new();
    // let mut args: Vec<String> = env::args().collect();
    let mut pid = 0;
    let mut command: Vec<String> = Vec::new();
    let mut init = false;
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("The client for force-zsh-alias");
        ap.refer(&mut pid)
            .add_option(&["-p", "--pid"], Store, "set pid");
        ap.refer(&mut init)
            .add_option(&["-i", "--init"], StoreTrue, "initialize with aliases");
        ap.refer(&mut command)
            .add_argument("command", Collect, "command to check");
        ap.parse_args_or_exit();
    }
    if init {
        send_aliases(&client, pid);
        std::process::exit(0);
    }
    else {
        send_command(client, &command.join(" "), &pid);
    }

}

// Sends a command to the server
fn send_command(client: Client, command: &String, pid: &usize) {
    let url: String = BASE_URL.to_string() + "commands";
    let body = PostCommand {
        pid: pid.clone(),
        command: command.clone()
    };
    let res_or_err = client.post(&url)
        .body(&serde_json::to_string(&body).unwrap())
        .send();
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
fn send_aliases(client: &Client, pid: usize) {
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
    let body = PostDeclarations {
        pid: pid.clone(),
        declarations: stdout.to_string().split("\n").map(|x| x.to_string()).collect()
    };
    let mut res = client.post(&url)
        .body(&serde_json::to_string(&body).unwrap())
        .send()
        .unwrap();
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
