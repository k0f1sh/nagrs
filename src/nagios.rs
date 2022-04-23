mod block;
pub mod object;

use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::path::Path;

use self::block::{Block, BlockType};
use self::object::{Host, Service};

#[derive(Debug)]
pub struct NagiosStatus {
    info: HashMap<String, String>,
    program: HashMap<String, String>,
    hosts: HashMap<String, Host>,
    services: HashMap<String, Vec<Service>>,
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
}
