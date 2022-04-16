use std::io::BufWriter;
use std::io::Write;
use std::path::Path;

use chrono::DateTime;
use chrono::Utc;

pub trait NagiosCmd {
    fn to_cmd_string(&self) -> String;
}

//////////////////////////////////
/// Cmd implementation

//////////////////////////////////
/// ENABLE_HOSTGROUP_HOST_CHECKS

pub struct EnableHostGroupHostChecks {
    hostgroup_name: String,
}

impl EnableHostGroupHostChecks {
    pub fn new(hostgroup_name: String) -> Self {
        EnableHostGroupHostChecks { hostgroup_name }
    }
}

impl NagiosCmd for EnableHostGroupHostChecks {
    fn to_cmd_string(&self) -> String {
        format!("ENABLE_HOSTGROUP_HOST_CHECKS;{}", self.hostgroup_name)
    }
}

//////////////////////////////////
/// DISABLE_HOSTGROUP_HOST_CHECKS

pub struct DisableHostGroupHostChecks {
    hostgroup_name: String,
}

impl DisableHostGroupHostChecks {
    pub fn new(hostgroup_name: String) -> Self {
        DisableHostGroupHostChecks { hostgroup_name }
    }
}

impl NagiosCmd for DisableHostGroupHostChecks {
    fn to_cmd_string(&self) -> String {
        format!("DISABLE_HOSTGROUP_HOST_CHECKS;{}", self.hostgroup_name)
    }
}

#[cfg(test)]
mod tests {
    use crate::cmd;

    use super::*;

    const TEST_DATETIME_STR: &str = "2022-03-21T01:00:00Z";

    fn written_string(cmd: Box<dyn NagiosCmd>) -> String {
        let datetime = DateTime::parse_from_rfc3339(TEST_DATETIME_STR)
            .unwrap()
            .with_timezone(&Utc);
        let mut buf = BufWriter::new(vec![]);
        let result = cmd::write_cmd_line(&cmd, datetime.timestamp(), &mut buf);

        match result {
            Err(_) => "".to_string(),
            Ok(_) => String::from_utf8(buf.into_inner().unwrap()).unwrap(),
        }
    }

    #[test]
    fn test_write_cmd_line() {
        let cmd = Box::new(EnableHostGroupHostChecks::new("localhost".to_string()));
        assert_eq!(
            written_string(cmd),
            "[1647824400] ENABLE_HOSTGROUP_HOST_CHECKS;localhost\n"
        );
    }
}
