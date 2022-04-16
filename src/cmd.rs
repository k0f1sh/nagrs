use crate::NagiosCmd;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;

use chrono::DateTime;
use chrono::Utc;

pub fn write_cmd_line<W: Write>(
    cmd: &Box<dyn NagiosCmd>,
    timestamp: i64,
    writer: &mut BufWriter<W>,
) -> std::io::Result<()> {
    let cmd_str = cmd.to_cmd_string();
    writer.write(format!("[{}] {}\n", timestamp, cmd_str).as_bytes())?;
    Ok(())
}

pub fn write_cmds<P: AsRef<Path>>(
    path: P,
    cmds: &Vec<Box<dyn NagiosCmd>>,
    datetime: &DateTime<Utc>,
) -> std::io::Result<()> {
    let timestamp = datetime.timestamp();
    let file = std::fs::OpenOptions::new().append(true).open(path)?;
    let mut writer = BufWriter::new(file);
    cmds.iter()
        .try_for_each(|cmd| write_cmd_line(cmd, timestamp, &mut writer))?;

    Ok(())
}
