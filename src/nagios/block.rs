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
