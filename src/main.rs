use std::env;

fn decode(message: &str) -> (String, String) {
    // TODO: check if message is valid
    match message.chars().next() {
        Some('i') => {
            match message.split_once("e") {
                Some((first_part, rest_part)) => {
                    match first_part.split_once("i") {
                        Some((_, number)) => {
                            // println!(" decode int -> number {} | rest_part {}", number, rest_part);
                            return (number.to_string(), rest_part.to_string());
                        }
                        None => {
                            panic!("Fail decode int. Missing i.");
                        }
                    }
                }
                None => {
                    panic!("Fail decode int. Missing e.");
                }
            }
        }
        Some('l') => {
            let mut decoded_message_list: Vec<String> = vec![];
            let (mut decoded_message, mut rest_of_message) = decode(&message[1..]);
            decoded_message_list.push(decoded_message);
            while rest_of_message != "".to_string() {
                (decoded_message, rest_of_message) = decode(&rest_of_message);
                if decoded_message == "".to_string() {
                    break;
                }
                decoded_message_list.push(decoded_message)
            }
            return (
                format!("[{}]", decoded_message_list.join(",")),
                rest_of_message.to_string(),
            );
        }
        Some('0'..='9') => {
            match message.split_once(":") {
                Some((number_part, text_part)) => {
                    if let Ok(char_len) = number_part.parse::<usize>() {
                        if char_len <= text_part.chars().count() {
                            let decoded_message =
                                format!("\"{}\"", &text_part[0..char_len]).to_string();
                            let rest_of_message = (&text_part[char_len..]).to_string();
                            // println!(" decode str -> number_part {} | text_part {} | rest_of_message {}", number_part, text_part, rest_of_message);
                            return (decoded_message, rest_of_message);
                        } else {
                            panic!("Fail decode str. Length mismatch.");
                        };
                    } else {
                        panic!("Fail decode str. Invalid number.");
                    }
                }
                None => {
                    // println!("{}",message);
                    panic!("Fail decode str.");
                }
            };
        }
        _ => return ("".to_string(), (&message[1..]).to_string()),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let action: &str = &args[1];
    let message: &str = &args[2];

    match action {
        "decode" => {
            let (decoded_message, _) = decode(message);
            println!("{}", decoded_message);
        }
        // TODO encode
        _ => panic!("Invalid action"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bencoded_string() {
        let test_cases = vec![
            ("5:hello", ((r#""hello""#).to_string(), "".to_string())),
            (
                "5:hello13432143124",
                ((r#""hello""#).to_string(), "13432143124".to_string()),
            ),
            (
                "15:123456789012345",
                ((r#""123456789012345""#).to_string(), "".to_string()),
            ),
            //("15:12345", ), // TODO: handle this case panic for now
        ];

        for (input, expected) in test_cases {
            assert_eq!(decode(input), expected);
        }
    }

    #[test]
    fn test_bencoded_int() {
        let test_cases = vec![
            ("i52e", (("52").to_string(), "".to_string())),
            ("i-52e", (("-52").to_string(), "".to_string())),
            (
                "i-123456789012345e",
                (("-123456789012345").to_string(), "".to_string()),
            ),
            ("i52esadw", (("52").to_string(), "sadw".to_string())),
        ];

        for (input, expected) in test_cases {
            assert_eq!(decode(input), expected);
        }
    }

    #[test]
    fn test_bencoded_list() {
        let test_cases = vec![
            (
                "l5:helloe",
                ((r#"["hello"]"#).to_string(), "".to_string()),
            ),
            (
                "l5:helloi52ee",
                ((r#"["hello",52]"#).to_string(), "".to_string()),
            ),
            (
                "l5:helloi52ee12345",
                ((r#"["hello",52]"#).to_string(), "12345".to_string()),
            ),
            (
                "l5:helloi52e5:helloe",
                ((r#"["hello",52,"hello"]"#).to_string(), "".to_string()),
            ),
            (
                "l5:helloi42el9:innerlisti-1eei52e5:halloe",
                (
                    (r#"["hello",42,["innerlist",-1],52,"hallo"]"#).to_string(),
                    "".to_string(),
                ),
            ),
        ];

        for (input, expected) in test_cases {
            assert_eq!(decode(input), expected);
        }
    }
}
