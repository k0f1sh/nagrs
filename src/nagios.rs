use std::collections::btree_map::Keys;
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

#[derive(Debug, PartialEq)]
enum ParseState {
    WithinBlock,
    Outside,
}

impl Block {
    fn parse_file<P: AsRef<Path>>(path: P) -> Result<Vec<Block>> {
        let file = File::open(path)?;
        let buf = io::BufReader::new(file);
        Block::parse(buf)
    }

    fn parse<R: Read>(buf: io::BufReader<R>) -> Result<Vec<Block>> {
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

pub type Result<T> = std::result::Result<T, ParseError>;

#[derive(Debug)]
pub struct ConvertHostError;

#[derive(Debug)]
pub enum ParseError {
    IoError(io::Error),
    UnexpectedLine(String),
    InvalidKeyValue(String),
    HostNameKeyNotExists,
    ConvertError(ConvertHostError),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::IoError(err) => write!(f, "{}", err),
            Self::UnexpectedLine(s) => write!(f, "unexpected line: {}", s),
            Self::InvalidKeyValue(s) => write!(f, "invalid line: {}", s),
            Self::HostNameKeyNotExists => write!(f, "host name key is not exists"),
            Self::ConvertError(_) => write!(f, "failed to convert"),
        }
    }
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::IoError(err) => Some(err),
            Self::UnexpectedLine(_) => None,
            Self::InvalidKeyValue(_) => None,
            Self::HostNameKeyNotExists => None,
            Self::ConvertError(err) => Some(err),
        }
    }
}

impl From<io::Error> for ParseError {
    fn from(err: io::Error) -> ParseError {
        ParseError::IoError(err)
    }
}

impl From<ConvertHostError> for ParseError {
    fn from(err: ConvertHostError) -> ParseError {
        ParseError::ConvertError(err)
    }
}

impl fmt::Display for ConvertHostError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed to convert to host")
    }
}

impl error::Error for ConvertHostError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

////////////////////////////////////
// nagios status

const HOST_NAME_KEY: &str = "host_name";
const NOTIFICATIONS_ENABLED_KEY: &str = "notifications_enabled";
const ACTIVE_CHECKS_ENABLED_KEY: &str = "active_checks_enabled";

type HostName = String;

#[derive(Debug, PartialEq, Clone)]
pub struct Host {
    host_name: HostName,
    notifications_enabled: bool,
    active_checks_enabled: bool,
    // TODO add fields as needed
}

impl Host {
    fn from_key_values(
        key_values: HashMap<String, String>,
    ) -> std::result::Result<Host, ConvertHostError> {
        let host_name = key_values.get(HOST_NAME_KEY).ok_or(ConvertHostError)?;
        let notifications_enabled = match key_values
            .get(NOTIFICATIONS_ENABLED_KEY)
            .ok_or(ConvertHostError)?
            .as_str()
        {
            "0" => Ok(false),
            "1" => Ok(true),
            _ => Err(ConvertHostError),
        }?;

        let active_checks_enabled = match key_values
            .get(ACTIVE_CHECKS_ENABLED_KEY)
            .ok_or(ConvertHostError)?
            .as_str()
        {
            "0" => Ok(false),
            "1" => Ok(true),
            _ => Err(ConvertHostError),
        }?;

        Ok(Host {
            host_name: host_name.to_owned(),
            notifications_enabled,
            active_checks_enabled,
        })
    }
}

#[derive(Debug)]
pub struct NagiosStatus {
    info: HashMap<String, String>,
    program: HashMap<String, String>,
    hosts: HashMap<HostName, Host>,
    services: HashMap<HostName, Vec<HashMap<String, String>>>,
    contacts: Vec<HashMap<String, String>>,
}

impl NagiosStatus {
    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<NagiosStatus> {
        let file = File::open(path)?;
        let buf = io::BufReader::new(file);
        let blocks = Block::parse(buf)?;
        Self::from_blocks(blocks)
    }

    fn from_blocks(blocks: Vec<Block>) -> Result<NagiosStatus> {
        let mut status = NagiosStatus {
            info: HashMap::new(),
            program: HashMap::new(),
            hosts: HashMap::new(),
            services: HashMap::new(),
            contacts: Vec::new(),
        };

        for block in blocks {
            match &block.block_type {
                BlockType::Info => {
                    status.info = block.key_values;
                }
                BlockType::Program => {
                    status.program = block.key_values;
                }
                BlockType::Host => {
                    let host = Host::from_key_values(block.key_values)?;
                    status.hosts.insert(host.host_name.to_owned(), host);
                }
                BlockType::Service => {
                    let host_name = block.key_values.get(HOST_NAME_KEY);
                    match host_name {
                        Some(host_name) => {
                            let host_services = status.services.get_mut(host_name);
                            match host_services {
                                Some(host_service) => host_service.push(block.key_values),
                                None => {
                                    status
                                        .services
                                        .insert(host_name.to_string(), vec![block.key_values]);
                                }
                            }
                        }
                        None => return Err(ParseError::HostNameKeyNotExists),
                    }
                }
                BlockType::Contact => status.contacts.push(block.key_values),
                _ => {}
            }
        }

        Ok(status)
    }

    pub fn get_info(&self) -> &HashMap<String, String> {
        &self.info
    }

    pub fn get_program(&self) -> &HashMap<String, String> {
        &self.program
    }

    pub fn get_host(&self, host_name: &str) -> Option<&Host> {
        self.hosts.get(host_name)
    }

    pub fn get_host_services(&self, host_name: &str) -> Option<&Vec<HashMap<String, String>>> {
        self.services.get(host_name)
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
        ]);
        let block = blocks.get(2).unwrap();
        assert_eq!(block.block_type, BlockType::Host);
        assert_eq!(block.key_values, expected);

        // services
        let expected = HashMap::from([
            ("host_name".to_string(), "web01".to_string()),
            ("service_description".to_string(), "PING".to_string()),
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

    #[test]
    fn parse_error_unexpected_block_name() {
        let unexpected_status_dat = r#"
            piyo {
                created=123456789
                version=9.99
            }
        "#;

        let buf = io::BufReader::new(unexpected_status_dat.as_bytes());
        let result = Block::parse(buf);

        match result {
            Err(ParseError::UnexpectedLine(s)) => {
                assert_eq!(s, "piyo {".to_string())
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn parse_error_invalid_key_value() {
        let invalid_status_dat = r#"
            hoststatus {
                piyo
            }
        "#;

        let buf = io::BufReader::new(invalid_status_dat.as_bytes());
        let result = Block::parse(buf);

        match result {
            Err(ParseError::InvalidKeyValue(s)) => {
                assert_eq!(s, "piyo".to_string())
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn nagios_status() {
        let buf = io::BufReader::new(STATUS_DAT.as_bytes());
        let blocks = Block::parse(buf).unwrap();
        let status = NagiosStatus::from_blocks(blocks).unwrap();

        assert_eq!(
            status.info,
            HashMap::from([
                ("created".to_string(), "123456789".to_string()),
                ("version".to_string(), "9.99".to_string()),
            ])
        );

        assert_eq!(
            status.program,
            HashMap::from([
                ("daemon_mode".to_string(), "1".to_string()),
                ("nagios_pid".to_string(), "99999".to_string()),
            ])
        );

        assert_eq!(
            status.hosts,
            HashMap::from([(
                "web01".to_string(),
                Host {
                    host_name: "web01".to_string(),
                    notifications_enabled: true,
                    active_checks_enabled: true,
                }
            )])
        );

        assert_eq!(
            status.services,
            HashMap::from([(
                "web01".to_string(),
                vec![HashMap::from([
                    ("host_name".to_string(), "web01".to_string()),
                    ("service_description".to_string(), "PING".to_string()),
                ]),],
            )])
        );

        assert_eq!(
            status.services,
            HashMap::from([(
                "web01".to_string(),
                vec![HashMap::from([
                    ("host_name".to_string(), "web01".to_string()),
                    ("service_description".to_string(), "PING".to_string()),
                ]),],
            )])
        );

        assert_eq!(
            status.contacts,
            vec![HashMap::from([
                ("contact_name".to_string(), "nagiosadmin".to_string()),
                ("modified_attributes".to_string(), "0".to_string()),
            ])]
        )
    }
}
