use crate::nagios::object::Host;
use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use nagios::cmd::NagiosCmd;
use nagios::object::Service;
use regex::Regex;
use std::collections::HashMap;
use std::io::BufWriter;
use std::path::Path;

use nagios::NagiosStatus;

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

    /// Get status

    fn load(&mut self) -> Result<()> {
        self.last_loaded = Utc::now();
        let status = NagiosStatus::parse_file(&self.status_file_path)?;
        self.status = Some(status);
        Ok(())
    }

    fn load_smartly(&mut self) -> Result<()> {
        let now = Utc::now();
        let diff = now - self.last_loaded;

        if self.status.is_none() || diff.num_seconds() >= self.max_cache_sec as i64 {
            self.load()?
        }

        Ok(())
    }

    pub fn get_info(&mut self) -> Result<HashMap<String, String>> {
        self.load_smartly()?;
        let status = self.status.as_ref().unwrap();
        Ok(status.get_info().clone())
    }

    pub fn find_host(&mut self, host_name: &str) -> Result<Option<Host>> {
        self.load_smartly()?;
        let status = self.status.as_ref().unwrap();
        Ok(status.get_host(host_name))
    }

    pub fn find_hosts_regex(&mut self, re: &Regex) -> Result<Vec<Host>> {
        self.load_smartly()?;
        let status = self.status.as_ref().unwrap();
        Ok(status.get_hosts_regex(re))
    }

    pub fn find_services(&mut self, host_name: &str) -> Result<Vec<Service>> {
        self.load_smartly()?;
        let status = self.status.as_ref().unwrap();
        Ok(status.get_host_services(host_name).unwrap_or(Vec::new()))
    }

    /// cmd
    pub fn write_cmds(&mut self, cmds: &Vec<Box<dyn NagiosCmd>>) -> std::io::Result<()> {
        let timestamp = Utc::now().timestamp();
        let file = std::fs::OpenOptions::new()
            .append(true)
            .open(&self.command_file_path)?;
        let mut writer = BufWriter::new(file);
        cmds.iter()
            .try_for_each(|cmd| nagios::cmd::write_cmd_line(cmd, timestamp, &mut writer))?;
        self.clear_cache();
        Ok(())
    }

    fn clear_cache(&mut self) -> () {
        self.status = None;
    }
}
