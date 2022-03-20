use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, Read};
use std::path::Path;
use std::{error, fmt};

type NagiosInfo = HashMap<String, String>;
type NagiosProgram = HashMap<String, String>;

type NagiosHost = HashMap<String, String>;
type NagiosHostName = String;

type NagiosService = HashMap<String, String>;

#[derive(Debug)]
pub struct NagiosStatus {
    pub info: NagiosInfo,
    pub program: NagiosProgram,
    pub hosts: HashMap<NagiosHostName, NagiosHost>,
    pub services: HashMap<NagiosHostName, Vec<NagiosService>>,
}

#[derive(Debug, PartialEq)]
enum BlockType {
    Info,
    Program,
    Host,
    Service,
}

#[derive(Debug, PartialEq)]
enum ParseState {
    WithinBlock(BlockType),
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
            .filter(|line| {
                if line.len() == 0 {
                    return false;
                };
                if line.chars().nth(0).unwrap() == '#' {
                    return false;
                };
                true
            });

        let mut info: NagiosInfo = HashMap::new();
        let mut program: NagiosProgram = HashMap::new();
        let mut hosts: HashMap<NagiosHostName, NagiosHost> = HashMap::new();
        let mut services: HashMap<NagiosHostName, Vec<NagiosService>> = HashMap::new();

        let mut tmp_host: NagiosHost = HashMap::new();
        let mut tmp_service: NagiosService = HashMap::new();

        let mut current_state: ParseState = ParseState::Outside;

        for line in lines {
            match &current_state {
                ParseState::Outside => {
                    match line.as_str() {
                        // start block
                        "info {" => current_state = ParseState::WithinBlock(BlockType::Info),
                        "programstatus {" => {
                            current_state = ParseState::WithinBlock(BlockType::Program)
                        }
                        "hoststatus {" => current_state = ParseState::WithinBlock(BlockType::Host),
                        "servicestatus {" => {
                            current_state = ParseState::WithinBlock(BlockType::Service)
                        }
                        _ => {
                            return Err(ParseError::UnexpectedBlockName(line.clone()));
                        }
                    }
                }
                ParseState::WithinBlock(block_type) => {
                    match (block_type, line.as_str()) {
                        // within block
                        (BlockType::Host, "}") => match tmp_host.get("host_name") {
                            Some(key) => {
                                let key = key.clone();
                                hosts.insert(key, tmp_host);
                                tmp_host = HashMap::new();
                                current_state = ParseState::Outside;
                            }
                            None => {
                                return Err(ParseError::HostNameDoesNotExist);
                            }
                        },
                        (BlockType::Service, "}") => {
                            let key = tmp_service.get("host_name").unwrap().clone();
                            let v = services.get_mut(&key);
                            match v {
                                None => {
                                    services.insert(key, vec![tmp_service]);
                                }
                                Some(v) => v.push(tmp_service),
                            }
                            tmp_service = HashMap::new();
                            current_state = ParseState::Outside;
                        }
                        (_, "}") => {
                            current_state = ParseState::Outside;
                        }
                        (block_type, s) => match s.split_once('=') {
                            Some((key, value)) => match block_type {
                                BlockType::Info => {
                                    info.insert(key.to_string(), value.to_string());
                                }
                                BlockType::Program => {
                                    program.insert(key.to_string(), value.to_string());
                                }
                                BlockType::Host => {
                                    tmp_host.insert(key.to_string(), value.to_string());
                                }
                                BlockType::Service => {
                                    tmp_service.insert(key.to_string(), value.to_string());
                                }
                            },
                            None => {
                                return Err(ParseError::InvalidKeyValue(s.to_string()));
                            }
                        },
                    }
                }
            }
        }

        Ok(NagiosStatus {
            info,
            program,
            hosts,
            services,
        })
    }
}

type Result<T> = std::result::Result<T, ParseError>;

#[derive(Debug)]
pub enum ParseError {
    IoError(io::Error),
    UnexpectedBlockName(String),
    HostNameDoesNotExist,
    InvalidKeyValue(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::IoError(err) => write!(f, "{}", err),
            Self::UnexpectedBlockName(s) => write!(f, "unexpected block name: {}", s),
            Self::HostNameDoesNotExist => write!(f, "host_name does not exist"),
            Self::InvalidKeyValue(s) => write!(f, "invalid line: {}", s),
        }
    }
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::IoError(err) => Some(err),
            Self::UnexpectedBlockName(_) => None,
            Self::HostNameDoesNotExist => None,
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

            hoststatus {
                host_name=web02
                state_type=1
                hoge=
            }

            servicestatus {
                host_name=web01
                service_description=PING
            }

            servicestatus {
                host_name=web01
                service_description=PONG
                a=b=c
            }
        "#;

        let buf = io::BufReader::new(status_dat.as_bytes());
        let nagios_status = NagiosStatus::parse(buf).unwrap();

        // info
        let expected = HashMap::from([
            ("created".to_string(), "123456789".to_string()),
            ("version".to_string(), "9.99".to_string()),
        ]);
        assert_eq!(nagios_status.info, expected);

        // program
        let expected = HashMap::from([
            ("daemon_mode".to_string(), "1".to_string()),
            ("nagios_pid".to_string(), "99999".to_string()),
        ]);
        assert_eq!(nagios_status.program, expected);

        // hosts
        let expected = HashMap::from([
            (
                "web01".to_string(),
                HashMap::from([
                    ("host_name".to_string(), "web01".to_string()),
                    ("state_type".to_string(), "1".to_string()),
                ]),
            ),
            (
                "web02".to_string(),
                HashMap::from([
                    ("host_name".to_string(), "web02".to_string()),
                    ("state_type".to_string(), "1".to_string()),
                    ("hoge".to_string(), "".to_string()),
                ]),
            ),
        ]);
        assert_eq!(nagios_status.hosts, expected);

        // services
        let expected = HashMap::from([(
            "web01".to_string(),
            vec![
                HashMap::from([
                    ("host_name".to_string(), "web01".to_string()),
                    ("service_description".to_string(), "PING".to_string()),
                ]),
                HashMap::from([
                    ("host_name".to_string(), "web01".to_string()),
                    ("service_description".to_string(), "PONG".to_string()),
                    ("a".to_string(), "b=c".to_string()),
                ]),
            ],
        )]);
        assert_eq!(nagios_status.services, expected);
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
            Err(ParseError::UnexpectedBlockName(s)) => {
                assert_eq!(s, "piyo {".to_string())
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn parse_error_host_name_does_not_exist() {
        let status_dat = r#"
            hoststatus {
                hoge=fuga
            }
        "#;

        let buf = io::BufReader::new(status_dat.as_bytes());
        let result = NagiosStatus::parse(buf);

        match result {
            Err(ParseError::HostNameDoesNotExist) => {
                assert!(true)
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
