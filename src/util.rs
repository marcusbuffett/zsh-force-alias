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
