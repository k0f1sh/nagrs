use nagrs_derive::NagiosCmd;
use std::io::BufWriter;
use std::io::Write;

pub trait NagiosCmd {
    fn to_cmd_string(&self) -> String;
}

pub fn write_cmd_line<W: Write>(
    cmd: &Box<dyn NagiosCmd>,
    timestamp: i64,
    writer: &mut BufWriter<W>,
) -> std::io::Result<()> {
    let cmd_str = cmd.to_cmd_string();
    writer.write(format!("[{}] {}\n", timestamp, cmd_str).as_bytes())?;
    Ok(())
}

//////////////////////////////////
/// Cmd implementation

//////////////////////////////////
/// ENABLE_HOSTGROUP_HOST_CHECKS

#[derive(Debug, NagiosCmd)]
pub struct EnableHostgroupHostChecks {
    pub hostgroup_name: String,
}

//////////////////////////////////
/// DISABLE_HOSTGROUP_HOST_CHECKS

#[derive(Debug, NagiosCmd)]
pub struct DisableHostgroupHostChecks {
    pub hostgroup_name: String,
}

//////////////////////////////////
/// ENABLE_HOST_CHECK

#[derive(Debug, NagiosCmd)]
pub struct EnableHostCheck {
    pub host_name: String,
}

//////////////////////////////////
/// DISABLE_HOST_CHECK

#[derive(Debug, NagiosCmd)]
pub struct DisableHostCheck {
    pub host_name: String,
}

//////////////////////////////////
/// ENABLE_HOST_NOTIFICATIONS

#[derive(Debug, NagiosCmd)]
pub struct EnableHostNotifications {
    pub host_name: String,
}

//////////////////////////////////
/// DISABLE_HOST_NOTIFICATIONS

#[derive(Debug, NagiosCmd)]
pub struct DisableHostNotifications {
    pub host_name: String,
}

//////////////////////////////////
/// ENABLE_SVC_CHECK

#[derive(Debug, NagiosCmd)]
pub struct EnableSvcCheck {
    pub host_name: String,
    pub service_description: String,
}

//////////////////////////////////
/// DISABLE_SVC_CHECK

#[derive(Debug, NagiosCmd)]
pub struct DisableSvcCheck {
    pub host_name: String,
    pub service_description: String,
}

//////////////////////////////////
/// ENABLE_SVC_NOTIFICATIONS

#[derive(Debug, NagiosCmd)]
pub struct EnableSvcNotifications {
    pub host_name: String,
    pub service_description: String,
}

//////////////////////////////////
/// DISABLE_SVC_NOTIFICATIONS

#[derive(Debug, NagiosCmd)]
pub struct DisableSvcNotifications {
    pub host_name: String,
    pub service_description: String,
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use chrono::Utc;
    use std::io::BufWriter;

    use super::*;

    const TEST_DATETIME_STR: &str = "2022-03-21T01:00:00Z";

    fn written_string(cmd: Box<dyn NagiosCmd>) -> String {
        let datetime = DateTime::parse_from_rfc3339(TEST_DATETIME_STR)
            .unwrap()
            .with_timezone(&Utc);
        let mut buf = BufWriter::new(vec![]);
        let result = write_cmd_line(&cmd, datetime.timestamp(), &mut buf);

        match result {
            Err(_) => "".to_string(),
            Ok(_) => String::from_utf8(buf.into_inner().unwrap()).unwrap(),
        }
    }

    struct TestCase {
        cmd: Box<dyn NagiosCmd>,
        expected: &'static str,
    }

    #[test]
    fn test_write_cmd_line() {
        let test_cases = vec![
            // ENABLE_HOSTGROUP_HOST_CHECKS
            TestCase {
                cmd: Box::new(EnableHostgroupHostChecks {
                    hostgroup_name: "localhost".to_string(),
                }),
                expected: "[1647824400] ENABLE_HOSTGROUP_HOST_CHECKS;localhost\n",
            },
            // DISABLE_HOSTGROUP_HOST_CHECKS
            TestCase {
                cmd: Box::new(DisableHostgroupHostChecks {
                    hostgroup_name: "localhost".to_string(),
                }),
                expected: "[1647824400] DISABLE_HOSTGROUP_HOST_CHECKS;localhost\n",
            },
            // ENABLE_HOST_CHECK
            TestCase {
                cmd: Box::new(EnableHostCheck {
                    host_name: "localhost".to_string(),
                }),
                expected: "[1647824400] ENABLE_HOST_CHECK;localhost\n",
            },
            // DISABLE_HOST_CHECK
            TestCase {
                cmd: Box::new(DisableHostCheck {
                    host_name: "localhost".to_string(),
                }),
                expected: "[1647824400] DISABLE_HOST_CHECK;localhost\n",
            },
            // ENABLE_HOST_NOTIFICATIONS
            TestCase {
                cmd: Box::new(EnableHostNotifications {
                    host_name: "localhost".to_string(),
                }),
                expected: "[1647824400] ENABLE_HOST_NOTIFICATIONS;localhost\n",
            },
            // DISABLE_HOST_NOTIFICATIONS
            TestCase {
                cmd: Box::new(DisableHostNotifications {
                    host_name: "localhost".to_string(),
                }),
                expected: "[1647824400] DISABLE_HOST_NOTIFICATIONS;localhost\n",
            },
            // ENABLE_SVC_CHECK
            TestCase {
                cmd: Box::new(EnableSvcCheck {
                    host_name: "localhost".to_string(),
                    service_description: "Current Load".to_string(),
                }),
                expected: "[1647824400] ENABLE_SVC_CHECK;localhost;Current Load\n",
            },
            // DISABLE_SVC_CHECK
            TestCase {
                cmd: Box::new(DisableSvcCheck {
                    host_name: "localhost".to_string(),
                    service_description: "Current Load".to_string(),
                }),
                expected: "[1647824400] DISABLE_SVC_CHECK;localhost;Current Load\n",
            },
            // ENABLE_SVC_NOTIFICATIONS
            TestCase {
                cmd: Box::new(EnableSvcNotifications {
                    host_name: "localhost".to_string(),
                    service_description: "Current Load".to_string(),
                }),
                expected: "[1647824400] ENABLE_SVC_NOTIFICATIONS;localhost;Current Load\n",
            },
            // DISABLE_SVC_NOTIFICATIONS
            TestCase {
                cmd: Box::new(DisableSvcNotifications {
                    host_name: "localhost".to_string(),
                    service_description: "Current Load".to_string(),
                }),
                expected: "[1647824400] DISABLE_SVC_NOTIFICATIONS;localhost;Current Load\n",
            },
        ];

        for test_case in test_cases {
            assert_eq!(written_string(test_case.cmd), test_case.expected);
        }
    }
}
