use chrono::DateTime;
use chrono::Utc;
use nagios::InvalidRegexError;
use nagios::NagiosError;
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;

use nagios::{Host, NagiosStatus, Service};

mod cmd;
pub mod nagios;

#[derive(Debug)]
pub struct Nagrs<P: AsRef<Path>> {
    command_file_path: P,
    status_file_path: P,
    status: Option<NagiosStatus>,
    last_loaded: DateTime<Utc>,
    max_cache_sec: usize,
}

impl<P: AsRef<Path>> Nagrs<P> {
    pub fn new(command_file_path: P, status_file_path: P, max_cache_sec: usize) -> Nagrs<P> {
        Nagrs {
            command_file_path,
            status_file_path,
            status: None,
            last_loaded: Utc::now(),
            max_cache_sec,
        }
    }

    fn load(&mut self) -> nagios::Result<()> {
        self.last_loaded = Utc::now();
        let status = NagiosStatus::parse_file(&self.status_file_path)?;
        self.status = Some(status);
        Ok(())
    }

    fn load_smartly(&mut self) -> nagios::Result<()> {
        let now = Utc::now();
        let diff = now - self.last_loaded;

        if self.status.is_none() || diff.num_seconds() >= self.max_cache_sec as i64 {
            self.load()?
        }

        Ok(())
    }

    pub fn get_info(&mut self) -> nagios::Result<HashMap<String, String>> {
        self.load_smartly()?;
        let status = self.status.as_ref().unwrap();
        Ok(status.get_info().clone())
    }

    pub fn find_host(&mut self, host_name: &str) -> nagios::Result<Option<Host>> {
        self.load_smartly()?;
        let status = self.status.as_ref().unwrap();
        Ok(status.get_host(host_name))
    }

    pub fn find_hosts_regex(&mut self, re: &Regex) -> nagios::Result<Vec<Host>> {
        self.load_smartly()?;
        let status = self.status.as_ref().unwrap();
        Ok(status.get_hosts_regex(re))
    }

    pub fn find_services(&mut self, host_name: &str) -> nagios::Result<Vec<Service>> {
        self.load_smartly()?;
        let status = self.status.as_ref().unwrap();
        Ok(status.get_host_services(host_name).unwrap_or(Vec::new()))
    }
}
