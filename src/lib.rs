use anyhow::Result;
use chrono::Utc;
use nagios::cmd::NagiosCmd;
use std::io::BufWriter;
use std::path::Path;

use nagios::NagiosStatus;

pub mod nagios;

#[derive(Debug)]
pub struct Nagrs<P: AsRef<Path>> {
    command_file_path: P,
    status_file_path: P,
}

impl<P: AsRef<Path>> Nagrs<P> {
    pub fn new(command_file_path: P, status_file_path: P) -> Nagrs<P> {
        Nagrs {
            command_file_path,
            status_file_path,
        }
    }

    pub fn parse(&self) -> Result<NagiosStatus> {
        NagiosStatus::parse_file(&self.status_file_path)
    }

    /// cmd
    pub fn write_cmds(&self, cmds: &Vec<Box<dyn NagiosCmd>>) -> std::io::Result<()> {
        let timestamp = Utc::now().timestamp();
        let file = std::fs::OpenOptions::new()
            .append(true)
            .open(&self.command_file_path)?;
        let mut writer = BufWriter::new(file);
        cmds.iter()
            .try_for_each(|cmd| nagios::cmd::write_cmd_line(cmd, timestamp, &mut writer))?;
        Ok(())
    }
}
