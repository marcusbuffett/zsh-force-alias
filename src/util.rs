pub fn index_of_substr(string: &str, substr: &str) -> Option<usize> {
    for (i, _) in string.chars().enumerate() {
        let mut matches = true;
        for (j, sub_ch) in substr.chars().enumerate() {
            if i + j >= string.len() {
                matches = false;
                break;
            }
            let chars_equal  = string.chars().nth(i+j).unwrap() == sub_ch;
            if  !chars_equal {
                matches = false;
                break;
            }
        }
        if matches {
            return Some(i)
        }
    }
    None
}

pub fn index_of_word(string: &str, substr: &str) -> Option<usize> {
    let i_opt = index_of_substr(&string, substr);
    if i_opt == None {
        return None;
    }
    let i = i_opt.unwrap();
    let preceding_ch_opt;
    // To avoid an overflow by trying to subtract from 0
    //
    // There's probably a better idiom for this
    if i == 0 {
        preceding_ch_opt = None;
    }
    else {
        preceding_ch_opt = string.chars().nth(i - 1);
    }
    let trailing_ch_opt = string.chars().nth(i + substr.len());
    let ch_opt_valid = |ch: &Option<char>| {
        *ch == None || ch.unwrap() == ' '
    };
    let both_valid = vec![preceding_ch_opt, trailing_ch_opt].iter()
        .all(|x| ch_opt_valid(x));
    if both_valid {
        return Some(i);
    }
    return None;
}

#[test]
fn index_of_substr_works() {
    assert_eq!(index_of_substr("Don't let your dreams be dreams",
                               "dreams"),
                               Some(15));
    assert_eq!(index_of_substr("Don't let your dreams be dreams",
                               "memes"),
                               None);
}

pub fn unquote_string(string: &String) -> String {
    let len = string.len();
    if len < 2 {
        return "".to_string();
    }
    match string.chars().next().unwrap() {
        '\'' | '"' => {
        }
        _ => {
            return string.clone();
        }
    }
    string.chars()
          .skip(1)
          .take(len-2)
          .collect::<String>()
}

#[test]
fn unquote_string_works() {
    assert_eq!(unquote_string(&"'42'".to_string()), "42".to_string());
    assert_eq!(unquote_string(&"42".to_string()), "42".to_string());
}
