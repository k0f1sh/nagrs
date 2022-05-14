use std::collections::HashMap;
use std::io::{self, BufRead, Read};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ParseError {
    #[error("invalid key value: {0}")]
    InvalidKeyValue(String),
    #[error("unexpected line: {0}")]
    UnexpectedLine(String),
    #[error("unexpected end of line")]
    UnexpectedEndOfLine,
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

struct Lines<'a> {
    iter: Box<dyn Iterator<Item = String> + 'a>,
}

impl<'a> Block {
    pub fn to_blocks<R: Read + 'a>(
        buf: io::BufReader<R>,
    ) -> impl Iterator<Item = Result<Block, ParseError>> + 'a {
        let iter = buf
            .lines()
            .filter(|r| r.is_ok())
            .map(|r| r.unwrap())
            .map(|line| line.trim().to_owned())
            .filter(|line| line.len() > 0)
            .filter(|line| line.chars().nth(0).unwrap() != '#');

        let lines = Lines {
            iter: Box::new(iter),
        };

        lines.into_iter()
    }
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

impl<'a> Iterator for Lines<'a> {
    type Item = Result<Block, ParseError>;

    fn next(&mut self) -> Option<Result<Block, ParseError>> {
        let mut tmp_block = Block::new();
        let mut current_state: ParseState = ParseState::Outside;
        let iter = &mut self.iter;

        for line in iter {
            match &current_state {
                ParseState::Outside => match select_block_type(line.as_str()) {
                    Ok(block_type) => {
                        tmp_block.block_type = block_type;
                        current_state = ParseState::WithinBlock;
                    }
                    Err(error) => return Some(Err(error)),
                },
                ParseState::WithinBlock => match line.as_str() {
                    "}" => {
                        return Some(Ok(tmp_block));
                    }
                    s => match s.split_once('=') {
                        Some((key, value)) => {
                            tmp_block
                                .key_values
                                .insert(key.to_string(), value.to_string());
                        }
                        None => {
                            return Some(Err(ParseError::InvalidKeyValue(s.to_string())));
                        }
                    },
                },
            }
        }

        if tmp_block.key_values.keys().count() > 0 {
            return Some(Err(ParseError::UnexpectedEndOfLine));
        }

        return None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_blocks() {
        let status_text = r#"
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

        let buf = io::BufReader::new(status_text.as_bytes());
        let blocks = Block::to_blocks(buf);
        assert_eq!(blocks.count(), 5)
    }

    #[test]
    fn test_to_blocks_error() {
        let status_text = r#"
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
        "#;

        let buf = io::BufReader::new(status_text.as_bytes());
        let blocks = Block::to_blocks(buf);
        let blocks = blocks.collect::<Result<Vec<_>, _>>();
        assert_eq!(blocks.is_err(), true);
    }

    #[test]
    fn test_to_blocks_error_unexpected_line() {
        let status_text = r#"
        unexpected_block {
            created=123456789
            version=9.99
        }
        "#;

        let buf = io::BufReader::new(status_text.as_bytes());
        let blocks = Block::to_blocks(buf);
        let blocks = blocks.collect::<Result<Vec<_>, _>>();
        assert_eq!(blocks.is_err(), true);
        assert_eq!(
            blocks.unwrap_err(),
            ParseError::UnexpectedLine("unexpected_block {".to_string())
        );
    }

    #[test]
    fn test_to_blocks_error_invalid_key_value() {
        let status_text = r#"
        hoststatus {
            created=123456789
            version=9.99
            error_line
        }
        "#;

        let buf = io::BufReader::new(status_text.as_bytes());
        let blocks = Block::to_blocks(buf);
        let blocks = blocks.collect::<Result<Vec<_>, _>>();
        assert_eq!(blocks.is_err(), true);
        assert_eq!(
            blocks.unwrap_err(),
            ParseError::InvalidKeyValue("error_line".to_string())
        );
    }

    #[test]
    fn test_to_blocks_unexpexted_end_of_line() {
        let status_text = r#"
        hoststatus {
            created=123456789
            version=9.99
        "#;

        let buf = io::BufReader::new(status_text.as_bytes());
        let blocks = Block::to_blocks(buf);
        let blocks = blocks.collect::<Result<Vec<_>, _>>();
        assert_eq!(blocks.is_err(), true);
        assert_eq!(blocks.unwrap_err(), ParseError::UnexpectedEndOfLine);
    }
}
