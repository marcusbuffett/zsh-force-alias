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

mod util;
mod alias;

// TODO
// Make hash of pid -> aliases

lazy_static! {
    static ref ALIASES: Mutex<Vec<alias::Alias>> =
        Mutex::new(Vec::new());
}

fn main() {
    let mut router = Router::new();
    router.get("/commands", list_commands);
    router.post("/commands", check_command);
    router.get("/aliases", list_aliases);
    router.post("/aliases", post_aliases);

    Iron::new(router).http("localhost:5671").unwrap();

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

    fn check_command(req: &mut Request) -> IronResult<Response> {
        let mut command = String::new();
        let _ = req.body.read_to_string(&mut command);
        let mut res_code = status::Ok;
        let mut aliases: Vec<alias::Alias> = Vec::new();
        let shortened = alias::shorten_command(&command, ALIASES.lock().unwrap().deref(), &mut aliases);
        let mut feedback = String::new();
        if shortened.len() != command.len() {
            res_code = status::BadRequest;
            let keystrokes_saved = command.len() - shortened.len();
            let mut feedback_lines = vec![
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

    fn list_aliases(_: &mut Request) -> IronResult<Response> {
        let aliases = ALIASES.lock().unwrap().clone();
        Ok(Response::with((status::Ok, format!("{:?}", aliases))))
    }

    fn list_commands(_req: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "blah blah")))
    }

    fn hello_world(_: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "Hello World!")))
    }

    Iron::new(hello_world).http("localhost:3000").unwrap();
    println!("On 3000");
}
