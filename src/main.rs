use clap::{Parser, Subcommand};
use serde::Deserialize;
use serde::Serialize;
use sha1::{Digest, Sha1};
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::result::Result;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand, Debug)]
enum Command {
    Decode { value: String },
    Info { torrent_filename: String },
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Torrent {
    announce: String,
    #[serde(skip)]
    #[serde(rename = "created by")]
    created_by: String,
    info: TorrentInfo,
    info_hash_sha1: String,
}
impl Torrent {
    pub fn from_bencode(value: &Bencode) -> Result<Self, String> {
        if let Bencode::Dict(dict) = value {
            let Some(announce) = dict.get("announce") else {
                return Err("announce not found".to_string());
            };
            let Some(created_by) = dict.get("created by") else {
                return Err("created by not found".to_string());
            };
            let Some(bencode_info) = dict.get("info") else {
                return Err("info not found".to_string());
            };
            let Bencode::Dict(info) = bencode_info else {
                return Err("info not found".to_string());
            };
            let Some(Bencode::Str(name)) = info.get("name") else {
                return Err("name not found in info".to_string());
            };
            let Some(Bencode::Int(piece_length)) = info.get("piece length") else {
                return Err("piece_length not found in info".to_string());
            };
            let Some(Bencode::RawStr(pieces)) = info.get("pieces") else {
                return Err("pieces not found in info".to_string());
            };
            let Some(Bencode::Int(length)) = info.get("length") else {
                return Err("length not found in info".to_string());
            };
            let mut hasher = Sha1::new();
            hasher.update(bencode_info.encode());
            let hash = format!("{:x}", hasher.finalize());

            let mut pieces_hash = Vec::<String>::new();
            let mut count = 0u32;
            let mut temp_string = String::new();
            for piece in pieces.iter() {
                match format!("{:x}", piece).len() {
                    1 => temp_string = format!("{}{}", temp_string, format!("0{:x}", piece)),
                    _ => temp_string = format!("{}{}", temp_string, format!("{:x}", piece)),
                }
                count += 1;
                if count == 20 {
                    pieces_hash.push(temp_string);
                    temp_string = "".to_string();
                    count = 0
                }
            }

            return Ok(Torrent {
                announce: announce.to_string(),
                created_by: created_by.to_string(),
                info: TorrentInfo {
                    name: name.to_string(),
                    piece_length: *piece_length,
                    pieces: pieces.clone(),
                    length: *length,
                    pieces_hash: pieces_hash,
                },
                info_hash_sha1: hash,
            });
        }
        Err("Bencode provided is not a dict".to_string())
    }
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
struct TorrentInfo {
    length: i64,
    name: String,
    #[serde(rename = "piece length")]
    piece_length: i64,
    pieces: Vec<u8>,
    pieces_hash: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Bencode {
    Int(i64),
    Str(String),
    RawStr(Vec<u8>),
    List(Vec<Bencode>),
    Dict(BTreeMap<String, Bencode>),
}
impl Bencode {
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Bencode::Int(int) => format!("i{}e", int).into_bytes(),
            Bencode::Str(str) => format!("{}:{}", str.len(), str).into_bytes(),
            Bencode::RawStr(vec) => {
                let mut prefix = format!("{}:", vec.len()).into_bytes();
                prefix.extend(vec);
                prefix
            }
            Bencode::List(vec) => {
                let mut byte_string = "l".to_string().into_bytes();
                for item in vec {
                    byte_string.extend(item.encode());
                }
                byte_string.push(b'e');
                byte_string
            }
            Bencode::Dict(hash_map) => {
                let mut byte_string = "d".to_string().into_bytes();
                for (key, value) in hash_map {
                    byte_string.extend(format!("{}:{}", key.len(), key).into_bytes());
                    byte_string.extend(value.encode());
                }
                byte_string.push(b'e');
                byte_string
            }
        }
    }
}

impl fmt::Display for Bencode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Bencode::Int(n) => write!(f, "{}", n),
            Bencode::Str(s) => write!(f, "{}", s),
            Bencode::RawStr(s) => write!(f, "{}", String::from_utf8_lossy(s)),
            Bencode::List(lst) => {
                write!(f, "[")?;
                for (i, item) in lst.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Bencode::Dict(dict) => {
                write!(f, "{{")?;
                for (i, (key, value)) in dict.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "\"{}\":{}", key, value)?;
                }
                write!(f, "}}")
            }
        }
    }
}

fn decode_integer(input: &[u8]) -> Result<(i64, usize), String> {
    if input.first() != Some(&b'i') {
        return Err("Integer must start with 'i'".to_string());
    }
    let end = input
        .iter()
        .position(|&b| b == b'e')
        .ok_or("Missing 'e' terminator")?;
    let num_str = std::str::from_utf8(&input[1..end]).map_err(|_| "Invalid UTF-8")?;
    let num = num_str.parse::<i64>().map_err(|_| "Invalid integer")?;
    Ok((num, end + 1))
}

fn decode_list(input: &[u8]) -> Result<(Vec<Bencode>, usize), String> {
    if input.first() != Some(&b'l') {
        return Err("List must start with 'l'".to_string());
    }

    let mut items = Vec::new();
    let mut pos = 1;

    while input.get(pos) != Some(&b'e') {
        let (item, consumed) = decode(&input[pos..])?;
        items.push(item);
        pos += consumed;
    }

    Ok((items, pos + 1))
}

use std::collections::BTreeMap;

fn decode_dict(input: &[u8]) -> Result<(BTreeMap<String, Bencode>, usize), String> {
    if input.first() != Some(&b'd') {
        return Err("Dict must start with 'd'".to_string());
    }

    let mut map = BTreeMap::new();
    let mut pos = 1;

    while input.get(pos) != Some(&b'e') {
        let (key, key_len) = decode_string(&input[pos..])?;
        pos += key_len;
        let (value, value_len) = decode(&input[pos..])?;
        pos += value_len;
        map.insert(key, value);
    }

    Ok((map, pos + 1))
}

fn decode_string(input: &[u8]) -> Result<(String, usize), String> {
    let colon_pos = input
        .iter()
        .position(|&b| b == b':')
        .ok_or("Missing colon")?;
    let len = std::str::from_utf8(&input[..colon_pos])
        .map_err(|_| "Invalid UTF-8 in string length")?
        .parse::<usize>()
        .map_err(|_| "Invalid length")?;

    let start = colon_pos + 1;
    let end = start + len;
    if end > input.len() {
        return Err("String data out of bounds".to_string());
    }

    let bytes = &input[start..end];

    match std::str::from_utf8(bytes) {
        Ok(s) => Ok(((s.to_string()), end)),
        Err(_) => Err("Invalid UTF-8 in string".to_string()),
    }
}
fn decode_raw_string(input: &[u8]) -> Result<(Vec<u8>, usize), String> {
    let colon_pos = input
        .iter()
        .position(|&b| b == b':')
        .ok_or("Missing colon")?;
    let len = std::str::from_utf8(&input[..colon_pos])
        .map_err(|_| "Invalid UTF-8 in string length")?
        .parse::<usize>()
        .map_err(|_| "Invalid length")?;

    let start = colon_pos + 1;
    let end = start + len;
    if end > input.len() {
        return Err("String data out of bounds".to_string());
    }

    let bytes = &input[start..end];

    Ok((bytes.to_vec(), end))
}

fn decode(input: &[u8]) -> Result<(Bencode, usize), String> {
    match input.first() {
        Some(b'i') => {
            let (n, len) = decode_integer(input)?;
            Ok((Bencode::Int(n), len))
        }
        Some(b'l') => {
            let (v, len) = decode_list(input)?;
            Ok((Bencode::List(v), len))
        }
        Some(b'd') => {
            let (m, len) = decode_dict(input)?;
            Ok((Bencode::Dict(m), len))
        }
        Some(b'0'..=b'9') => {
            if let Ok((s, len)) = decode_string(input) {
                Ok((Bencode::Str(s), len))
            } else {
                let (raw_s, len) = decode_raw_string(input)?;
                Ok((Bencode::RawStr(raw_s), len))
            }
        }
        _ => Err("Unknown type prefix".to_string()),
    }
}

fn parse_torrent_file(filename: &str) -> Result<Torrent, String> {
    let file = File::open(filename).map_err(|e| format!("Failed to open file: {}", e))?;
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    let _ = reader
        .read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read file: {}", e));

    let Ok((bencode, _)) = decode(&buffer) else {
        return Err("Failed to decode torrent file".to_string());
    };
    let parsed = Torrent::from_bencode(&bencode).expect("Failed to parse Torrent");

    Ok(parsed)
}

fn main() {
    let args = Args::parse();

    match args.command {
        Command::Decode { value } => match decode(value.as_bytes()) {
            Ok((bencode, _)) => println!("{}", bencode),
            Err(err) => eprintln!("Error: {}", err),
        },
        Command::Info { torrent_filename } => match parse_torrent_file(&torrent_filename) {
            Ok(torrent) => {
                println!("Tracker URL: {}", torrent.announce);
                println!("Length: {}", torrent.info.length);
                println!("Info Hash: {}", torrent.info_hash_sha1);
                println!("Piece Length: {}", torrent.info.piece_length);
                println!("Piece Hashes:");
                for piece_hash in torrent.info.pieces_hash.iter() {
                    println!("{}", piece_hash);
                }
            }
            Err(err) => eprintln!("Error: {}", err),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bencoded_string() {
        let test_cases = vec![
            (b"5:hello" as &[u8], Bencode::Str("hello".to_string())),
            (b"5:hello13432143124", Bencode::Str("hello".to_string())),
            (
                b"15:123456789012345",
                Bencode::Str("123456789012345".to_string()),
            ),
        ];

        for (input, expected) in test_cases {
            let (result, _rest) = decode(input).expect("Should decode");
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_bencoded_int() {
        let test_cases = vec![
            (b"i52e" as &[u8], Bencode::Int(52)),
            (b"i-52e", Bencode::Int(-52)),
            (b"i-123456789012345e", Bencode::Int(-123456789012345)),
            (b"i52esadw", Bencode::Int(52)),
        ];

        for (input, expected) in test_cases {
            let (result, _rest) = decode(input).expect("Should decode");
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_bencoded_list() {
        let test_cases = vec![
            (
                b"l5:helloe" as &[u8],
                Bencode::List(vec![Bencode::Str("hello".to_string())]),
            ),
            (
                b"l5:helloi52ee",
                Bencode::List(vec![Bencode::Str("hello".to_string()), Bencode::Int(52)]),
            ),
            (
                b"l5:helloi52ee12345",
                Bencode::List(vec![Bencode::Str("hello".to_string()), Bencode::Int(52)]),
            ),
            (
                b"l5:helloi52e5:helloe",
                Bencode::List(vec![
                    Bencode::Str("hello".to_string()),
                    Bencode::Int(52),
                    Bencode::Str("hello".to_string()),
                ]),
            ),
            (
                b"l5:helloi42el9:innerlisti-1eei52e5:halloe",
                Bencode::List(vec![
                    Bencode::Str("hello".to_string()),
                    Bencode::Int(42),
                    Bencode::List(vec![
                        Bencode::Str("innerlist".to_string()),
                        Bencode::Int(-1),
                    ]),
                    Bencode::Int(52),
                    Bencode::Str("hallo".to_string()),
                ]),
            ),
        ];

        for (input, expected) in test_cases {
            let (result, _rest) = decode(input).expect("Should decode");
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_bencoded_dict() {
        let test_cases = vec![
            (b"d3:foo3:bar5:helloi52ee" as &[u8], {
                let mut dict = BTreeMap::new();
                dict.insert("foo".to_string(), Bencode::Str("bar".to_string()));
                dict.insert("hello".to_string(), Bencode::Int(52));
                Bencode::Dict(dict)
            }),
            (b"de", Bencode::Dict(BTreeMap::new())),
            (b"d4:spam4:eggse", {
                let mut dict = BTreeMap::new();
                dict.insert("spam".to_string(), Bencode::Str("eggs".to_string()));
                Bencode::Dict(dict)
            }),
            (b"d3:numi123e3:str5:hello4:nestd3:key5:valueee", {
                let mut nested = BTreeMap::new();
                nested.insert("key".to_string(), Bencode::Str("value".to_string()));

                let mut dict = BTreeMap::new();
                dict.insert("num".to_string(), Bencode::Int(123));
                dict.insert("str".to_string(), Bencode::Str("hello".to_string()));
                dict.insert("nest".to_string(), Bencode::Dict(nested));
                Bencode::Dict(dict)
            }),
            (b"d1:ad1:bd1:ci1eeee", {
                let mut level3 = BTreeMap::new();
                level3.insert("c".to_string(), Bencode::Int(1));
                let mut level2 = BTreeMap::new();
                level2.insert("b".to_string(), Bencode::Dict(level3));
                let mut level1 = BTreeMap::new();
                level1.insert("a".to_string(), Bencode::Dict(level2));
                Bencode::Dict(level1)
            }),
            (b"d4:listl3:one3:two5:threee3:numi99ee", {
                let mut dict = BTreeMap::new();
                dict.insert(
                    "list".to_string(),
                    Bencode::List(vec![
                        Bencode::Str("one".to_string()),
                        Bencode::Str("two".to_string()),
                        Bencode::Str("three".to_string()),
                    ]),
                );
                dict.insert("num".to_string(), Bencode::Int(99));
                Bencode::Dict(dict)
            }),
            (b"d1:xi0e1:yi-42ee", {
                let mut dict = BTreeMap::new();
                dict.insert("x".to_string(), Bencode::Int(0));
                dict.insert("y".to_string(), Bencode::Int(-42));
                Bencode::Dict(dict)
            }),
        ];

        for (input, expected) in test_cases {
            let (result, _rest) = decode(input).expect("Should decode");
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_encode() {
        assert_eq!(Bencode::Str("hello".to_string()).encode(), b"5:hello");
        assert_eq!(
            Bencode::Str("123456789012345".to_string()).encode(),
            b"15:123456789012345"
        );
        assert_eq!(Bencode::Int(52).encode(), b"i52e");
        assert_eq!(Bencode::Int(-52).encode(), b"i-52e");
        assert_eq!(
            Bencode::Int(-123456789012345).encode(),
            b"i-123456789012345e"
        );
        assert_eq!(
            Bencode::List(vec![Bencode::Str("hello".to_string())]).encode(),
            b"l5:helloe"
        );
        assert_eq!(
            {
                let mut dict = BTreeMap::new();
                dict.insert("foo".to_string(), Bencode::Str("bar".to_string()));
                dict.insert("hello".to_string(), Bencode::Int(52));
                Bencode::Dict(dict).encode()
            },
            b"d3:foo3:bar5:helloi52ee"
        );
    }

    #[test]
    fn test_file_dict() {
        let dict = include_bytes!("../sample.torrent");
        let Ok((bencode, _)) = decode(dict) else {
            panic!("Failed to decode torrent file");
        };
        let parsed = Torrent::from_bencode(&bencode).expect("Failed to parse Torrent");

        assert_eq!(
            parsed.announce,
            "http://bittorrent-test-tracker.codecrafters.io/announce"
        );
        assert_eq!(parsed.info.length, 92063);
        assert_eq!(
            parsed.info_hash_sha1,
            "d69f91e6b2ae4c542468d1073a71d4ea13879a7f"
        );
    }
}
