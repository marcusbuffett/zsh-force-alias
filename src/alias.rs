use util;

#[derive(Debug, Clone, PartialEq)]
pub enum AliasScope {
    Normal,
    Global
}

#[derive(Clone, Debug, PartialEq)]
pub struct Alias {
    scope: AliasScope,
    alias: String,
    command: String
}

// TODO: make private, call from within shorten
// Lengthens a command by replacing aliases with commands
pub fn lengthen_command(command: &String, aliases: &Vec<Alias>, used_aliases: &mut Vec<Alias>) -> String {
    let mut alias_matches: Vec<Alias> = Vec::new();
    for alias in aliases {
        match alias.alias_used_by(&command) {
            Some(_) => {
                alias_matches.push(alias.clone())
            }
            _ => {
                continue
            }
        }
    }
    let most_efficient_alias = alias_matches
        .iter()
        .max_by_key(|alias|
            alias.command.len() - alias.alias.len());
    match most_efficient_alias {
        Some(alias) => {
            let lengthened = alias.reverse_use_in(command);
            if lengthened.len() > command.len() {
                used_aliases.push(alias.clone());
                let alias_idx = aliases.iter().position(|ref other_alias| other_alias == &alias).unwrap();
                let mut remaining_aliases = aliases.clone();
                remaining_aliases.remove(alias_idx);
                return lengthen_command(&lengthened, &remaining_aliases, used_aliases);
            }
        }
        None => {
        }
    }
    return command.clone();
}

#[test]
fn lengthen_command_works() {
    let g_alias = Alias {
        alias: "g".to_string(),
        command: "git".to_string(),
        scope: AliasScope::Normal
    };
    assert_eq!(lengthen_command(&"g status".to_string(), &mut vec![g_alias], &mut vec![]), "git status".to_string());
}

// Shortens a command by using aliases
pub fn shorten_command(command: &String, aliases: &Vec<Alias>, used_aliases: &mut Vec<Alias>) -> String {
    let mut alias_matches: Vec<Alias> = Vec::new();
    for alias in aliases {
        match alias.used_by(&command) {
            Some(_) => {
                alias_matches.push(alias.clone())
            }
            _ => {
                continue
            }
        }
    }
    let most_efficient_alias = alias_matches
        .iter()
        .max_by_key(|alias|
            alias.command.len() - alias.alias.len());

    match most_efficient_alias {
        Some(alias) => {
            let shortened = alias.use_in(command);
            if shortened.len() < command.len() {
                used_aliases.push(alias.clone());
                return shorten_command(&shortened, aliases, used_aliases);
            }
        }
        None => {
        }
    }
    return command.clone();
}

// Parses alias declarations of the form:
//
// alias x="xargs"
// alias g="git"
// alias -g G="| grep"
//
// and turns them into a Vec<Alias>
pub fn parse_alias_declarations(alias_declarations: Vec<&str>) -> Vec<Alias> {
    let mut aliases : Vec<Alias> = Vec::new();
    for declaration in alias_declarations {
        // NOTE: According to IEEE 1003.1, env variables
        // cannot contain '='. I'm going to assume
        // that applies to aliases too. 
        //
        // http://pubs.opengroup.org/onlinepubs/000095399/basedefs/xbd_chap08.html
        let left_and_right : Vec<String> = declaration.split("=").map(|x| x.to_string()).collect();
        if left_and_right.len() < 2 {
            continue;
        }
        let left_side = left_and_right.get(0).unwrap();
        let right_side = left_and_right
            .iter()
            // Skip left-hand side
            .skip(1)
            // Clone strings because #join is only implemented for Vec<String>
            .map(|x| x.clone())
            .collect::<Vec<String>>()
            .join("=");
        let scope;
        let alias;
        if left_side.contains("alias -g") {
            scope = AliasScope::Global;
            alias = left_side.split(" ").nth(2).unwrap().to_string();
        }
        else {
            scope = AliasScope::Normal;
            alias = left_side.split(" ").nth(1).unwrap().to_string();
        }
        let command: String = util::unquote_string(&right_side);
        let unquoted_command = util::unquote_string(&command.to_string());
        aliases.push(Alias {
            scope: scope,
            alias: alias,
            command: unquoted_command
        });
    }
    aliases
}

impl Alias {
    fn used_by(&self, command: &str) -> Option<usize> {
        let idx_opt = util::index_of_word(command, &self.command);
        if idx_opt.is_none() {
            return None;
        }
        let idx = idx_opt.unwrap();
        if idx == 0 || self.scope == AliasScope::Global {
            return Some(idx);
        }
        return None;
    }

    // TODO: dry up this and the one above, and rename
    fn alias_used_by(&self, command: &str) -> Option<usize> {
        let idx_opt = util::index_of_word(command, &self.alias);
        if idx_opt.is_none() {
            return None;
        }
        let idx = idx_opt.unwrap();
        if idx == 0 || self.scope == AliasScope::Global {
            return Some(idx);
        }
        return None;
    }


    pub fn use_in(&self, command: &String) -> String {
        //While the command contains the alias command string:
        //  Find the location of the alias command string
        //  Replace with alias
        let mut command: String = command.clone();
        let mut contained = self.used_by(&command);
        while contained != None {
            // NOTE: dry this up
            let index = contained.unwrap();
            let before_alias = command.chars().take(index).collect::<String>(); // .chain(alias.command.chars()).collect::<String>();
            let after_alias: String = command.chars().skip(index+self.command.len()).collect();
            command = before_alias + &self.alias + &after_alias;
            contained = self.used_by(&command);
        }
        command
    }

    pub fn reverse_use_in(&self, command: &String) -> String {
        //While the command contains the alias command string:
        //  Find the location of the alias command string
        //  Replace with alias
        let mut command: String = command.clone();
        // TODO: rename to _opt
        let mut contained = self.alias_used_by(&command);
        if contained != None {
            // NOTE: dry this up
            let index = contained.unwrap();
            let before_alias = command.chars().take(index).collect::<String>(); // .chain(alias.command.chars()).collect::<String>();
            let after_alias: String = command.chars().skip(index+self.alias.len()).collect();
            command = before_alias + &self.command + &after_alias;
        }
        command
    }

    pub fn fmt_for_feedback(&self) -> String {
        let mut formatted = format!("{} -> {}", self.alias, self.command);
        if self.scope == AliasScope::Global {
            formatted = formatted + " (GLOBAL) ";
        }
        formatted
    }
}

#[test]
fn used_by_works() {
    let gst_alias = Alias {
        scope: AliasScope::Normal,
        command: "git status".to_string(),
        alias: "gst".to_string()
    };
    let grep_alias = Alias {
        scope: AliasScope::Global,
        command: "| grep".to_string(),
        alias: "G".to_string()
    };
    let git_alias = Alias {
        scope: AliasScope::Global,
        command: "| grep".to_string(),
        alias: "G".to_string()
    };
    assert!(gst_alias.used_by("h git status -uno").is_none());
    assert!(git_alias.used_by("git-shell").is_none());
    assert!(grep_alias.used_by("git status | grep changed").is_some());
    assert!(grep_alias.used_by("git status | ag grepping ").is_none());
}

#[test]
fn use_in_works() {
    let alias = Alias {
        scope: AliasScope::Normal,
        command: "git status".to_string(),
        alias: "gst".to_string()
    };
    assert_eq!(alias.used_by("git status -uno"), Some(0))
}

#[test]
fn parse_alias_declarations_works() {
    let pu_test = ("alias pu=pushd", Alias {
        scope: AliasScope::Normal,
        alias: "pu".to_string(),
        command: "pushd".to_string()
    });
    let g_test = ("alias -g G='| grep'", Alias {
        scope: AliasScope::Global,
        alias: "G".to_string(),
        command: "| grep".to_string()
    });
    let grep_test = ("alias grep='grep --color=auto'", Alias {
        scope: AliasScope::Normal,
        alias: "grep".to_string(),
        command: "grep --color=auto".to_string()
    });
    let test_cases = vec![pu_test, g_test, grep_test];
    let test_declaration = |declaration: &String, desired: &Alias| {
        let aliases = parse_alias_declarations(declaration.split("\n").collect());
        assert!(aliases.len() != 0);
        assert_eq!(aliases.get(0).unwrap(), desired);
    };
    for test_case in test_cases {
        let (declaration, desired) = test_case;
        test_declaration(&declaration.to_string(), &desired);
    }
}
