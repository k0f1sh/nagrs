use std::collections::HashMap;
use std::io::{self, BufRead, Read};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ParseError {
    #[error("invalid key value: {0}")]
    InvalidKeyValue(String),
    #[error("unexpected line: {0}")]
    UnexpectedLine(String),
}

#[derive(Debug, PartialEq)]
pub enum BlockType {
    Info,
    Program,
    Host,
    Service,
    Contact,
    Unkown,
}

#[derive(Debug, PartialEq)]
pub struct Block {
    pub block_type: BlockType,
    pub key_values: HashMap<String, String>,
}

impl Block {
    fn new() -> Self {
        Block {
            block_type: BlockType::Unkown,
            key_values: HashMap::new(),
        }
    }
}

#[derive(Debug, PartialEq)]
enum ParseState {
    WithinBlock,
    Outside,
}

impl Block {
    pub fn parse<R: Read>(buf: io::BufReader<R>) -> Result<Vec<Block>, ParseError> {
        let lines = buf
            .lines()
            .filter(|r| r.is_ok())
            .map(|r| r.unwrap())
            .map(|line| line.trim().to_owned())
            .filter(|line| line.len() > 0)
            .filter(|line| line.chars().nth(0).unwrap() != '#');

        let mut blocks: Vec<Block> = vec![];
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
                        blocks.push(tmp_block);
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

        Ok(blocks)
    }

    fn select_block_type(line: &str) -> Result<BlockType, ParseError> {
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

#[cfg(test)]
mod tests {
    use super::*;

    const STATUS_DAT: &str = r#"
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
        notifications_enabled=1
        active_checks_enabled=1
        passive_checks_enabled=1
        obsess=1
        event_handler_enabled=1
        flap_detection_enabled=1
    }

    servicestatus {
        host_name=web01
        service_description=PING
        notifications_enabled=1
        active_checks_enabled=1
        passive_checks_enabled=1
        check_command=hoge
        obsess=1
        event_handler_enabled=1
        flap_detection_enabled=1

    }

    contactstatus {
        contact_name=nagiosadmin
        modified_attributes=0
        }
    
    "#;

    #[test]
    fn parse() {
        let buf = io::BufReader::new(STATUS_DAT.as_bytes());
        let blocks = Block::parse(buf).unwrap();

        // info
        let expected = HashMap::from([
            ("created".to_string(), "123456789".to_string()),
            ("version".to_string(), "9.99".to_string()),
        ]);
        let block = blocks.get(0).unwrap();
        assert_eq!(block.block_type, BlockType::Info);
        assert_eq!(block.key_values, expected);

        // program
        let expected = HashMap::from([
            ("daemon_mode".to_string(), "1".to_string()),
            ("nagios_pid".to_string(), "99999".to_string()),
        ]);
        let block = blocks.get(1).unwrap();
        assert_eq!(block.block_type, BlockType::Program);
        assert_eq!(block.key_values, expected);

        // host
        let expected = HashMap::from([
            ("host_name".to_string(), "web01".to_string()),
            ("notifications_enabled".to_string(), "1".to_string()),
            ("active_checks_enabled".to_string(), "1".to_string()),
            ("passive_checks_enabled".to_string(), "1".to_string()),
            ("obsess".to_string(), "1".to_string()),
            ("event_handler_enabled".to_string(), "1".to_string()),
            ("flap_detection_enabled".to_string(), "1".to_string()),
        ]);
        let block = blocks.get(2).unwrap();
        assert_eq!(block.block_type, BlockType::Host);
        assert_eq!(block.key_values, expected);

        // services
        let expected = HashMap::from([
            ("host_name".to_string(), "web01".to_string()),
            ("service_description".to_string(), "PING".to_string()),
            ("notifications_enabled".to_string(), "1".to_string()),
            ("active_checks_enabled".to_string(), "1".to_string()),
            ("passive_checks_enabled".to_string(), "1".to_string()),
            ("check_command".to_string(), "hoge".to_string()),
            ("obsess".to_string(), "1".to_string()),
            ("event_handler_enabled".to_string(), "1".to_string()),
            ("flap_detection_enabled".to_string(), "1".to_string()),
        ]);
        let block = blocks.get(3).unwrap();
        assert_eq!(block.block_type, BlockType::Service);
        assert_eq!(block.key_values, expected);

        // contacts
        let expected = HashMap::from([
            ("contact_name".to_string(), "nagiosadmin".to_string()),
            ("modified_attributes".to_string(), "0".to_string()),
        ]);
        let block = blocks.get(4).unwrap();
        assert_eq!(block.block_type, BlockType::Contact);
        assert_eq!(block.key_values, expected);
    }
}
