use std::{collections::HashMap, error, fmt, path::Path};

use nagios::NagiosStatus;

mod cmd;
pub mod nagios;

#[derive(Debug)]
pub struct Nagrs<P: AsRef<Path>> {
    command_file_path: P,
    status_file_path: P,
    status: Option<NagiosStatus>,
}

#[derive(Debug, Clone)]
pub struct StatusNotLoadedError;

impl fmt::Display for StatusNotLoadedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "nagios status not loaded")
    }
}

impl error::Error for StatusNotLoadedError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

impl<P: AsRef<Path>> Nagrs<P> {
    pub fn new(command_file_path: P, status_file_path: P) -> Nagrs<P> {
        Nagrs {
            command_file_path,
            status_file_path,
            status: None,
        }
    }

    pub fn load(&mut self) -> nagios::Result<()> {
        let status = NagiosStatus::parse_file(&self.status_file_path)?;
        self.status = Some(status);
        Ok(())
    }

    pub fn find_host(
        &self,
        host_name: &str,
    ) -> Result<Option<HashMap<String, String>>, StatusNotLoadedError> {
        if self.status.is_none() {
            return Err(StatusNotLoadedError);
        }

        let status = self.status.as_ref().unwrap();
        Ok(status.get_host(host_name).map(|h| h.clone()))
    }
}
