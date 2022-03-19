use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, Read};
use std::path::Path;

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

// TODO エラーをちゃんと独自のものに

impl NagiosStatus {
    pub fn parse_file<P: AsRef<Path>>(path: P) -> io::Result<NagiosStatus> {
        let file = File::open(path)?;
        let buf = io::BufReader::new(file);
        Ok(NagiosStatus::parse(buf))
    }

    fn parse<R: Read>(buf: io::BufReader<R>) -> NagiosStatus {
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
                        "program {" => current_state = ParseState::WithinBlock(BlockType::Program),
                        "host {" => current_state = ParseState::WithinBlock(BlockType::Host),
                        "service {" => current_state = ParseState::WithinBlock(BlockType::Service),
                        _ => {
                            // TODO return Error
                            panic!()
                        }
                    }
                }
                ParseState::WithinBlock(block_type) => {
                    match (block_type, line.as_str()) {
                        // within block
                        (BlockType::Host, "}") => {
                            // TODO エラー処理
                            let key = tmp_host.get("host_name").unwrap().clone();
                            hosts.insert(key, tmp_host);
                            tmp_host = HashMap::new();
                            current_state = ParseState::Outside;
                        }
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
                        (block_type, s) => {
                            let (key, value) = s
                                .split_once('=')
                                // TODO ちゃんとエラー返す
                                .expect(format!("failed to parse line: line={}", s).as_str());
                            match block_type {
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
                            }
                        }
                    }
                }
            }
        }

        NagiosStatus {
            info,
            program,
            hosts,
            services,
        }
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

            program {
                daemon_mode=1
                nagios_pid=99999
            }

            host {
                host_name=web01
                state_type=1
            }

            host {
                host_name=web02
                state_type=1
                hoge=
            }

            service {
                host_name=web01
                service_description=PING
            }

            service {
                host_name=web01
                service_description=PONG
                a=b=c
            }
        "#;

        let buf = io::BufReader::new(status_dat.as_bytes());
        let nagios_status = NagiosStatus::parse(buf);

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
}
