use std::env;

fn decode_string(char_len: i32, text_part: &str) -> (String, String) {
    let char_len = char_len as usize;
    let decoded_message = &text_part[0..char_len];
    let quotation_marks: &str = "\"";
    return (format!("{}{}{}", quotation_marks, decoded_message, quotation_marks), "".to_string());
}

fn decode_integer(message: &str) -> (String, String) {
    let length: usize = message.chars().count() as usize;
    let decoded_message = &message[1..=length-2];
    return (decoded_message.to_string(), "".to_string());
}
fn decode_list(message: &str) -> (String, String) {
    let length: usize = message.chars().count() as usize;
    let mut _inner_message: &str = &message[1..=length-2];
    let decoded_message: String = "".to_string();
    return (format!("{}{}{}", "[", decoded_message, "]"), "".to_string());
}

fn decode(message: &str) -> String {
    // TODO: check if message is valid
    // TODO: dont use unwrap
    let first_char: char = message.chars().next().unwrap();
    match first_char {
        '0'..='9' => {
            let message_parts: Vec<&str> = match message.split_once(":") {
                Some((first, second)) => vec![first, second],
                None => panic!("This is impossible, util is now!"),
            };        
            let (decoded_message, _) = decode_string(message_parts[0].parse::<i32>().unwrap(), &message_parts[1]);
            return decoded_message.to_string();
    
        }
        'i' => {
            let (decoded_message, _) = decode_integer(message);
            return decoded_message.to_string();
    
        }
        'l' => {
            let (decoded_message, _) = decode_list(message);
            return decoded_message.to_string();
    
        }
        _ => return "".to_string()
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let action: &str = &args[1];
    let message: &str = &args[2];

    match action {
        "decode" => {
            let decoded_message = decode(message);
            println!("{}", decoded_message);
        }
        // TODO encode
        _ => panic!("Invalid action"),
    }

}
