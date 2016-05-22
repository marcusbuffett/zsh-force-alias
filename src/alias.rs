use util;

#[derive(Debug, Clone, PartialEq)]
pub enum AliasScope {
    Normal,
    Global
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Alias {
    scope: AliasScope,
    alias: String,
    command: String
}

pub fn shorten_command(command: &String, aliases: &Vec<Alias>, used_aliases: &mut Vec<Alias>) -> String {
    let mut alias_matches: Vec<Alias> = Vec::new();
    for alias in aliases {
        match alias.used_by(&command) {
            Some(x) => {
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
    // Find all aliases that match command
    // Use alias with the longest expanded command
    // Replace the expanded command with the alias
    // Recurse
}

pub fn parse_alias_declarations(alias_declarations: Vec<&str>) -> Vec<Alias> {
    let mut aliases : Vec<Alias> = Vec::new();
    for declaration in alias_declarations {
        // NOTE: According to IEEE 1003.1, env variables
        // cannot contain '='. I'm going to assume
        // that applies to aliases too. 
        //
        // http://pubs.opengroup.org/onlinepubs/000095399/basedefs/xbd_chap08.html
        let left_and_right : Vec<String> = declaration.split("=").map(|x| x.to_string()).collect();
        if left_and_right.len() != 2 {
            continue;
        }
        let mut left_side = left_and_right.get(0).unwrap();
        let mut right_side = left_and_right
            .iter()
            // Skip left-hand side
            .skip(1)
            // Clone strings because #join is only implemented for Vec<String>
            .map(|x| x.clone())
            .collect::<Vec<String>>()
            .join("");
        let mut scope = AliasScope::Normal;
        let mut alias = String::new();
        if left_side.contains("alias -g") {
            scope = AliasScope::Global;
            alias = left_side.split(" ").nth(2).unwrap().to_string();
        }
        else {
            scope = AliasScope::Normal;
            alias = left_side.split(" ").nth(1).unwrap().to_string();
        }
        let mut command: String = util::unquote_string(&right_side);
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
        let i_opt = util::index_of_substr(&command, &self.command);
        if i_opt == None {
            return None;
        }
        let mut i = i_opt.unwrap();
        let mut preceding_ch_opt = Some(' ');
        // To avoid an overflow by trying to subtract from 0
        //
        // There's probably a better idiom for this
        if i == 0 {
            preceding_ch_opt = None;
        }
        else {
            preceding_ch_opt = command.chars().nth(i - 1);
        }
        let trailing_ch_opt = command.chars().nth(i + self.command.len());
        let ch_opt_valid = |ch: &Option<char>| {
            *ch == None || ch.unwrap() == ' '
        };
        let both_valid = vec![preceding_ch_opt, trailing_ch_opt].iter()
            .any(|x| ch_opt_valid(x));
        if both_valid {
            if i == 0 || self.scope == AliasScope::Global {
                return Some(i);
            }
            return None;
        }
        None
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
    let input = "alias pu=pushd\nalias -g G='| grep'".to_string();
    let declarations = input.split("\n").collect();
    let aliases = parse_alias_declarations(declarations);
    assert_eq!(aliases.len(), 2);
    let alias_pu = Alias {
        scope: AliasScope::Normal,
        alias: "pu".to_string(),
        command: "pushd".to_string()
    };
    let alias_g = Alias {
        scope: AliasScope::Global,
        alias: "G".to_string(),
        command: "| grep".to_string()
    };
    let desired_result = vec![alias_pu, alias_g];
    assert_eq!(aliases, desired_result);
}
