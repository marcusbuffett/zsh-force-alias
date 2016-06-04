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

// Import other files (modules)
mod util;
mod alias;

// TODO
// Make hash of pid -> aliases

lazy_static! {
    // Declares a static mutex holding a vector of aliases
    static ref ALIASES: Mutex<Vec<alias::Alias>> =
        Mutex::new(Vec::new());
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
        let alias_declarations : Vec<&str> = (&body_str).split("\n").collect();
        let new_aliases = alias::parse_alias_declarations(alias_declarations);
        let mut old_aliases = ALIASES.lock().unwrap();
        for new_alias in new_aliases {
            if !old_aliases.contains(&new_alias) {
                old_aliases.push(new_alias);
            }
        }
        Ok(Response::with((status::Ok, "Upload successful")))
    }

    // Endpoint to check a command
    fn check_command(req: &mut Request) -> IronResult<Response> {
        // Declare a mutable string and read the contents of
        // the body into it
        let mut command = String::new();
        let _ = req.body.read_to_string(&mut command);
        let mut res_code = status::Ok;
        let mut aliases: Vec<alias::Alias> = Vec::new();
        // Lengthen the command
        // ex: gst -uno -> git status -uno
        let lengthened = alias::lengthen_command(&command, ALIASES.lock().unwrap().deref(), &mut aliases);
        // Shorten the command
        // ex: git status -uno -> gsuno
        let shortened = alias::shorten_command(&lengthened, ALIASES.lock().unwrap().deref(), &mut aliases);
        let mut feedback = String::new();
        // If the shortened command is, in fact, shorter,
        // then return a BadRequest code
        if shortened.len() != command.len() {
            res_code = status::BadRequest;
            let keystrokes_saved = command.len() - shortened.len();
            let mut feedback_lines = vec![
                "".to_string(),
                "I'm sorry Dave, I can't let you do that.".to_string(),
                "".to_string(),
                format!("You could save {} keystrokes with:", keystrokes_saved),
                format!("> {}", shortened),
                "".to_string(),
                "Relevant aliases:".to_string()
            ];
            for alias in aliases {
                feedback_lines.push(alias.fmt_for_feedback());
            }
            feedback = feedback_lines.join("\n");
        }
        Ok(Response::with((res_code, feedback)))
    }

    // Endpoint to list aliases
    fn list_aliases(_: &mut Request) -> IronResult<Response> {
        let aliases = ALIASES.lock().unwrap().clone();
        Ok(Response::with((status::Ok, format!("{:?}", aliases))))
    }

    // Endpoint to list previous commands
    // Not yet implemented
    fn list_commands(_req: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "")))
    }
}
