use std::io::BufWriter;
use std::io::Write;

use chrono::DateTime;
use chrono::Utc;

pub trait NagiosCmd {
    fn to_cmd_string(&self) -> String;
}

pub fn write_cmd<W: Write>(
    cmd: &Box<dyn NagiosCmd>,
    datetime: &DateTime<Utc>,
    writer: &mut BufWriter<W>,
) -> std::io::Result<()> {
    let cmd_str = cmd.to_cmd_string();
    let timestamp = datetime.timestamp();
    writer.write(format!("[{}] {}\n", timestamp, cmd_str).as_bytes())?;
    Ok(())
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

    pub fn new_box(hostgroup_name: String) -> Box<dyn NagiosCmd> {
        Box::new(Self::new(hostgroup_name))
    }
}

impl NagiosCmd for EnableHostGroupHostChecks {
    fn to_cmd_string(&self) -> String {
        format!("ENABLE_HOSTGROUP_HOST_CHECKS;{}", self.hostgroup_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_cmd() {
        let cmd = EnableHostGroupHostChecks::new_box("localhost".to_string());
        let datetime = DateTime::parse_from_rfc3339("2022-03-21T01:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let mut buf = BufWriter::new(vec![]);
        let result = write_cmd(&cmd, &datetime, &mut buf);

        assert_eq!(result.is_ok(), true);

        let s = String::from_utf8(buf.into_inner().unwrap()).unwrap();
        assert_eq!(s, "[1647824400] ENABLE_HOSTGROUP_HOST_CHECKS;localhost\n");
    }
}
