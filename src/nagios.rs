use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, Read};
use std::path::Path;
use std::{error, fmt};

#[derive(Debug, PartialEq)]
enum BlockType {
    Info,
    Program,
    Host,
    Service,
    Contact,
    Unkown,
}

#[derive(Debug, PartialEq)]
struct Block {
    block_type: BlockType,
    key_values: HashMap<String, String>,
}

impl Block {
    fn new() -> Self {
        Block {
            block_type: BlockType::Unkown,
            key_values: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct NagiosStatus {
    blocks: Vec<Block>,
}

#[derive(Debug, PartialEq)]
enum ParseState {
    WithinBlock,
    Outside,
}

impl NagiosStatus {
    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<NagiosStatus> {
        let file = File::open(path)?;
        let buf = io::BufReader::new(file);
        NagiosStatus::parse(buf)
    }

    fn parse<R: Read>(buf: io::BufReader<R>) -> Result<NagiosStatus> {
        let lines = buf
            .lines()
            .filter(|r| r.is_ok())
            .map(|r| r.unwrap())
            .map(|line| line.trim().to_owned())
            .filter(|line| line.len() > 0)
            .filter(|line| line.chars().nth(0).unwrap() != '#');

        let mut status = NagiosStatus { blocks: Vec::new() };
        let mut tmp_block = Block::new();

        let mut current_state: ParseState = ParseState::Outside;

        for line in lines {
            match &current_state {
                ParseState::Outside => match Self::select_block_type(line.as_str()) {
                    Ok(block_type) => {
                        tmp_block.block_type = block_type;
                        current_state = ParseState::WithinBlock;
                    }
                    Err(error) => return Err(error),
                },
                ParseState::WithinBlock => match line.as_str() {
                    "}" => {
                        status.blocks.push(tmp_block);
                        tmp_block = Block::new();
                        current_state = ParseState::Outside;
                    }
                    s => match s.split_once('=') {
                        Some((key, value)) => {
                            tmp_block
                                .key_values
                                .insert(key.to_string(), value.to_string());
                        }
                        None => {
                            return Err(ParseError::InvalidKeyValue(s.to_string()));
                        }
                    },
                },
            }
        }

        Ok(status)
    }

    fn select_block_type(line: &str) -> Result<BlockType> {
        match line {
            "info {" => Ok(BlockType::Info),
            "programstatus {" => Ok(BlockType::Program),
            "hoststatus {" => Ok(BlockType::Host),
            "servicestatus {" => Ok(BlockType::Service),
            "contactstatus {" => Ok(BlockType::Contact),
            _ => Err(ParseError::UnexpectedLine(line.to_string())),
        }
    }
}

type Result<T> = std::result::Result<T, ParseError>;

#[derive(Debug)]
pub enum ParseError {
    IoError(io::Error),
    UnexpectedLine(String),
    InvalidKeyValue(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::IoError(err) => write!(f, "{}", err),
            Self::UnexpectedLine(s) => write!(f, "unexpected line: {}", s),
            Self::InvalidKeyValue(s) => write!(f, "invalid line: {}", s),
        }
    }
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::IoError(err) => Some(err),
            Self::UnexpectedLine(_) => None,
            Self::InvalidKeyValue(_) => None,
        }
    }
}

impl From<io::Error> for ParseError {
    fn from(err: io::Error) -> ParseError {
        ParseError::IoError(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let status_dat = r#"
            info {
                created=123456789
                version=9.99
            }

            programstatus {
                daemon_mode=1
                nagios_pid=99999
            }

            hoststatus {
                host_name=web01
                state_type=1
            }

            servicestatus {
                host_name=web01
                service_description=PING
            }

            contactstatus {
                contact_name=nagiosadmin
                modified_attributes=0
                }
            
            "#;

        let buf = io::BufReader::new(status_dat.as_bytes());
        let nagios_status = NagiosStatus::parse(buf).unwrap();

        // info
        let expected = HashMap::from([
            ("created".to_string(), "123456789".to_string()),
            ("version".to_string(), "9.99".to_string()),
        ]);
        let block = nagios_status.blocks.get(0).unwrap();
        assert_eq!(block.block_type, BlockType::Info);
        assert_eq!(block.key_values, expected);

        // program
        let expected = HashMap::from([
            ("daemon_mode".to_string(), "1".to_string()),
            ("nagios_pid".to_string(), "99999".to_string()),
        ]);
        let block = nagios_status.blocks.get(1).unwrap();
        assert_eq!(block.block_type, BlockType::Program);
        assert_eq!(block.key_values, expected);

        // host
        let expected = HashMap::from([
            ("host_name".to_string(), "web01".to_string()),
            ("state_type".to_string(), "1".to_string()),
        ]);
        let block = nagios_status.blocks.get(2).unwrap();
        assert_eq!(block.block_type, BlockType::Host);
        assert_eq!(block.key_values, expected);

        // services
        let expected = HashMap::from([
            ("host_name".to_string(), "web01".to_string()),
            ("service_description".to_string(), "PING".to_string()),
        ]);
        let block = nagios_status.blocks.get(3).unwrap();
        assert_eq!(block.block_type, BlockType::Service);
        assert_eq!(block.key_values, expected);

        // contacts
        let expected = HashMap::from([
            ("contact_name".to_string(), "nagiosadmin".to_string()),
            ("modified_attributes".to_string(), "0".to_string()),
        ]);
        let block = nagios_status.blocks.get(4).unwrap();
        assert_eq!(block.block_type, BlockType::Contact);
        assert_eq!(block.key_values, expected);
    }

    #[test]
    fn parse_error_unexpected_block_name() {
        let status_dat = r#"
            piyo {
                created=123456789
                version=9.99
            }
        "#;

        let buf = io::BufReader::new(status_dat.as_bytes());
        let result = NagiosStatus::parse(buf);

        match result {
            Err(ParseError::UnexpectedLine(s)) => {
                assert_eq!(s, "piyo {".to_string())
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn parse_error_invalid_key_value() {
        let status_dat = r#"
            hoststatus {
                piyo
            }
        "#;

        let buf = io::BufReader::new(status_dat.as_bytes());
        let result = NagiosStatus::parse(buf);

        match result {
            Err(ParseError::InvalidKeyValue(s)) => {
                assert_eq!(s, "piyo".to_string())
            }
            _ => assert!(false),
        }
    }
}
