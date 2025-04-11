use std::env;

fn decode(mut message: String) -> String {
    // TODO: check if message is valid
    let mut decoded_message: Vec<String> = vec![];
    let mut rest_of_message: String = "irrelevant".to_string();
    while rest_of_message != "" {
        // TODO: dont use unwrap
        let first_char: char = message.chars().next().unwrap();
        match first_char {
            'i' => match message.split_once("e") {
                Some((main_part, rest_part)) => {
                    let length: usize = main_part.chars().count() as usize;
                    decoded_message.push((main_part[1..=length - 1]).to_string());
                    rest_of_message = rest_part.to_string();
                }
                None => {
                    rest_of_message = "".to_string();
                }
            },
            'l' => {
                let length: usize = message.chars().count() as usize;
                let mut _inner_message: &str = &message[1..=length - 2];
                let temp_decoded_message = decode(_inner_message.to_string());
                decoded_message.push(format!("\"[{}]\"", temp_decoded_message));
                rest_of_message = "".to_string();
            }
            _ => {
                match message.split_once(":") {
                    Some((number_part, text_part)) => {
                        let char_len = number_part.parse::<i32>().unwrap() as usize;
                        if char_len <= text_part.len() {
                            decoded_message
                                .push(format!("\"{}\"", &text_part[0..char_len]).to_string());
                            rest_of_message = (&text_part[char_len..]).to_string();
                        } else {
                            panic!("Invalid message 1");
                        };
                    }
                    None => {
                        rest_of_message = "".to_string();
                    }
                };
            }
        }
        message = rest_of_message.clone();
    }
    let final_message = decoded_message.join(",");
    return final_message.to_string();
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let action: &str = &args[1];
    let message: &str = &args[2];

    match action {
        "decode" => {
            let decoded_message = decode(message.to_string());
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
            ("5:hello", r#""hello""#),
            ("5:hello13432143124", r#""hello""#),
            ("15:123456789012345", r#""123456789012345""#),
            // ("15:12345", ""), TODO: handle this case
        ];

        for (input, expected) in test_cases {
            assert_eq!(decode(input.to_string()), expected.to_string());
        }
    }

    #[test]
    fn test_bencoded_int() {
        let test_cases = vec![
            ("i52e", "52"),
            ("i-52e", "-52"),
            ("i-123456789012345e", "-123456789012345"),
            ("i52esadw", "52"),
        ];

        for (input, expected) in test_cases {
            assert_eq!(decode(input.to_string()), expected.to_string());
        }
    }

    #[test]
    fn test_bencoded_list() {
        let test_cases = vec![
            ("l5:helloe", r#""["hello"]""#),
            ("l5:helloi52ee", r#""["hello",52]""#),
            ("l5:helloi52ee12345", r#""["hello",52]""#),
            ("l5:helloi52e5:helloe", r#""["hello",52,"hello"]""#),
            // ("l5:hellol9:innerlistei52e5:helloe", r#""["hello",52,["innerlist"],52,"hello"]""#), TODO: handle this case
        ];

        for (input, expected) in test_cases {
            assert_eq!(decode(input.to_string()), expected.to_string());
        }
    }
}
