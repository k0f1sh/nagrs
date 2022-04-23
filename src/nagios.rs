mod block;
pub mod cmd;
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
