use std::path::Path;

use nagios::NagiosStatus;

mod cmd;
pub mod nagios;

#[derive(Debug)]
pub struct Nagrs<P: AsRef<Path>> {
    command_file_path: P,
    status_file_path: P,
    status: Option<NagiosStatus>,
}

impl<P: AsRef<Path>> Nagrs<P> {
    pub fn new(command_file_path: P, status_file_path: P) -> Nagrs<P> {
        Nagrs {
            command_file_path,
            status_file_path,
            status: None,
        }
    }
}
