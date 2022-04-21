mod object;

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, Read};
use std::path::Path;
use std::{error, fmt};

use regex::Regex;

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
    // fn parse_file<P: AsRef<Path>>(path: P) -> Result<Vec<Block>> {
    //     let file = File::open(path)?;
    //     let buf = io::BufReader::new(file);
    //     Block::parse(buf)
    // }

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
                            return Err(NagiosError::InvalidKeyValue(s.to_string()));
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
            _ => Err(NagiosError::UnexpectedLine(line.to_string())),
        }
    }
}

pub type Result<T> = std::result::Result<T, NagiosError>;

#[derive(Debug)]
pub struct ConvertHostError;

#[derive(Debug)]
pub struct InvalidRegexError;

#[derive(Debug)]
pub enum NagiosError {
    IoError(io::Error),
    UnexpectedLine(String),
    InvalidKeyValue(String),
    HostNameKeyNotExists,
    ConvertError(ConvertHostError),
}

impl fmt::Display for NagiosError {
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

impl error::Error for NagiosError {
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

impl From<io::Error> for NagiosError {
    fn from(err: io::Error) -> NagiosError {
        NagiosError::IoError(err)
    }
}

impl From<ConvertHostError> for NagiosError {
    fn from(err: ConvertHostError) -> NagiosError {
        NagiosError::ConvertError(err)
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

pub type HostName = String;

#[derive(Debug, PartialEq, Clone)]
pub struct Host {
    pub host_name: HostName,
    pub active_checks_enabled: bool,
    pub passive_checks_enabled: bool,
    pub obsess: bool,
    pub event_handler_enabled: bool,
    pub flap_detection_enabled: bool,
    pub notifications_enabled: bool,
    // TODO add fields as needed
}

fn get_bool_value(
    key: &str,
    key_values: &HashMap<String, String>,
) -> std::result::Result<bool, ConvertHostError> {
    match key_values.get(key).ok_or(ConvertHostError)?.as_str() {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(ConvertHostError),
    }
}

impl TryFrom<HashMap<String, String>> for Host {
    type Error = NagiosError;

    fn try_from(key_values: HashMap<String, String>) -> std::result::Result<Self, Self::Error> {
        let host_name = key_values.get("host_name").ok_or(ConvertHostError)?;
        let active_checks_enabled = get_bool_value("active_checks_enabled", &key_values)?;
        let passive_checks_enabled = get_bool_value("passive_checks_enabled", &key_values)?;
        let obsess = get_bool_value("obsess", &key_values)?;
        let event_handler_enabled = get_bool_value("event_handler_enabled", &key_values)?;
        let flap_detection_enabled = get_bool_value("flap_detection_enabled", &key_values)?;
        let notifications_enabled = get_bool_value("notifications_enabled", &key_values)?;

        Ok(Host {
            host_name: host_name.to_owned(),
            active_checks_enabled,
            passive_checks_enabled,
            obsess,
            event_handler_enabled,
            flap_detection_enabled,
            notifications_enabled,
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Service {
    pub host_name: HostName,
    pub check_command: String,
    pub service_description: String,
    pub active_checks_enabled: bool,
    pub passive_checks_enabled: bool,
    pub obsess: bool,
    pub event_handler_enabled: bool,
    pub flap_detection_enabled: bool,
    pub notifications_enabled: bool,
    // TODO add fields as needed
}

impl TryFrom<HashMap<String, String>> for Service {
    type Error = NagiosError;

    fn try_from(key_values: HashMap<String, String>) -> std::result::Result<Self, Self::Error> {
        let host_name = key_values.get("host_name").ok_or(ConvertHostError)?;
        let service_description = key_values
            .get("service_description")
            .ok_or(ConvertHostError)?;

        let check_command = key_values.get("check_command").ok_or(ConvertHostError)?;

        let active_checks_enabled = get_bool_value("active_checks_enabled", &key_values)?;
        let passive_checks_enabled = get_bool_value("passive_checks_enabled", &key_values)?;
        let obsess = get_bool_value("obsess", &key_values)?;
        let event_handler_enabled = get_bool_value("event_handler_enabled", &key_values)?;
        let flap_detection_enabled = get_bool_value("flap_detection_enabled", &key_values)?;
        let notifications_enabled = get_bool_value("notifications_enabled", &key_values)?;

        Ok(Service {
            host_name: host_name.to_owned(),
            service_description: service_description.to_owned(),
            check_command: check_command.to_owned(),
            active_checks_enabled,
            passive_checks_enabled,
            obsess,
            event_handler_enabled,
            flap_detection_enabled,
            notifications_enabled,
        })
    }
}

#[derive(Debug)]
pub struct NagiosStatus {
    info: HashMap<String, String>,
    program: HashMap<String, String>,
    hosts: HashMap<HostName, Host>,
    services: HashMap<HostName, Vec<Service>>,
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
                    let host = Host::try_from(block.key_values)?;
                    status.hosts.insert(host.host_name.to_owned(), host);
                }
                BlockType::Service => {
                    let service = Service::try_from(block.key_values)?;
                    let host_services = status.services.get_mut(&service.host_name);
                    match host_services {
                        Some(host_service) => host_service.push(service),
                        None => {
                            status
                                .services
                                .insert(service.host_name.to_string(), vec![service]);
                        }
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

    pub fn get_host(&self, host_name: &str) -> Option<Host> {
        self.hosts.get(host_name).map(|host| host.clone())
    }

    pub fn get_host_services(&self, host_name: &str) -> Option<Vec<Service>> {
        self.services
            .get(host_name)
            .map(|services| services.clone())
    }

    pub fn get_hosts_regex(&self, re: &Regex) -> Vec<Host> {
        self.hosts
            .iter()
            .filter(|(host_name, _)| re.is_match(&host_name))
            .map(|(_, host)| host.clone())
            .collect()
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
            Err(NagiosError::UnexpectedLine(s)) => {
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
            Err(NagiosError::InvalidKeyValue(s)) => {
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
                    passive_checks_enabled: true,
                    obsess: true,
                    event_handler_enabled: true,
                    flap_detection_enabled: true,
                }
            )])
        );

        assert_eq!(
            status.services,
            HashMap::from([(
                "web01".to_string(),
                vec![Service {
                    host_name: "web01".to_string(),
                    service_description: "PING".to_string(),
                    notifications_enabled: true,
                    active_checks_enabled: true,
                    passive_checks_enabled: true,
                    obsess: true,
                    event_handler_enabled: true,
                    flap_detection_enabled: true,
                    check_command: "hoge".to_string(),
                }],
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
