#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

#[macro_use] extern crate lazy_static;

extern crate iron;
extern crate router;
extern crate rustc_serialize;
extern crate bodyparser;
extern crate serde_json;

use iron::prelude::*;
use iron::status;
use router::Router;
use std::io::Read;
use std::sync::Mutex;
use std::ops::Deref;
use std::collections::HashMap;
use std::str::FromStr;

// Import other files (modules)
mod util;
mod alias;
mod request_types;

use request_types::{PostDeclarations, PostCommand};

// TODO
// Make hash of pid -> aliases

lazy_static! {
    // Declares a static mutex holding a vector of aliases
    static ref PID_TO_ALIASES: Mutex<HashMap<usize, Vec<alias::Alias>>> =
        Mutex::new(HashMap::new());
}

// Entry point
fn main() {
    let mut router = Router::new();

    // Define routes and corresponding handlers
    router.get("/commands", list_commands);
    router.post("/commands", check_command);
    router.get("/aliases", list_aliases);
    router.post("/aliases", post_aliases);

    // Returns an Option, depending on whether the port bind succeeded
    let router_opt = Iron::new(router).http("localhost:5571");
    if router_opt.is_err() {
        std::process::exit(0);
    }
    router_opt.unwrap();

    // Endpoint for aliases to be posted
    fn post_aliases(req: &mut Request) -> IronResult<Response> {
        let mut body_str = String::new();
        let _ = req.body.read_to_string(&mut body_str);
        let declarations_post: PostDeclarations = serde_json::from_str(&body_str).unwrap();
        let new_aliases = alias::parse_alias_declarations(declarations_post.declarations);
        let mut pid_to_aliases = PID_TO_ALIASES.lock().unwrap();
        // TODO: clean
        // let mut aliases = pid_to_aliases.get(declarations_post.get(declarations_post.pid).unwrap()).unwrap_or(Vec::new());
        pid_to_aliases.insert(declarations_post.pid, Vec::new());

        for new_alias in new_aliases {
            pid_to_aliases.get_mut(&declarations_post.pid).unwrap().push(new_alias);
        }
        Ok(Response::with((status::Ok, "Upload successful")))
    }

    // Endpoint to check a command
    fn check_command(req: &mut Request) -> IronResult<Response> {
        // Declare a mutable string and read the contents of
        // the body into it
        let mut body_str = String::new();
        let _ = req.body.read_to_string(&mut body_str);
        let body: PostCommand = serde_json::from_str(&body_str).unwrap();
        let mut res_code = status::Ok;
        let mut used_aliases: Vec<alias::Alias> = Vec::new();
        // Lengthen the command
        // ex: gst -uno -> git status -uno
        // TODO: rename other stuff to body
        let pid_to_aliases = PID_TO_ALIASES.lock().unwrap();
        let empty_aliases = Vec::new();
        let aliases = pid_to_aliases.get(&body.pid).unwrap_or(&empty_aliases);
        let lengthened = alias::lengthen_command(&body.command, aliases, &mut used_aliases);
        // Shorten the command
        // ex: git status -uno -> gsuno
        let shortened = alias::shorten_command(&lengthened, aliases, &mut used_aliases);
        let mut feedback = String::new();
        // If the shortened command is, in fact, shorter,
        // then return a BadRequest code
        if shortened.len() != body.command.len() {
            res_code = status::BadRequest;
            let keystrokes_saved = body.command.len() - shortened.len();
            let mut feedback_lines = vec![
                "".to_string(),
                "I'm sorry Dave, I can't let you do that.".to_string(),
                "".to_string(),
                format!("You could save {} keystrokes with:", keystrokes_saved),
                format!("> {}", shortened),
                "".to_string(),
                "Relevant aliases:".to_string()
            ];
            for alias in used_aliases {
                feedback_lines.push(alias.fmt_for_feedback());
            }
            feedback = feedback_lines.join("\n");
        }
        Ok(Response::with((res_code, feedback)))
    }

    // Endpoint to list aliases
    fn list_aliases(req: &mut Request) -> IronResult<Response> {
        let mut body_str = String::new();
        let _ = req.body.read_to_string(&mut body_str);
        let empty_aliases = Vec::new();
        let pid_to_aliases = PID_TO_ALIASES.lock().unwrap();
        let aliases = pid_to_aliases.get(&usize::from_str(&body_str).unwrap()).unwrap_or(&empty_aliases);
        Ok(Response::with((status::Ok, format!("{:?}", aliases))))
    }

    // Endpoint to list previous commands
    // Not yet implemented
    fn list_commands(_req: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "")))
    }
}
