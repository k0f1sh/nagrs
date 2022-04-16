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
        Self { hostgroup_name }
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
        Self { hostgroup_name }
    }
}

impl NagiosCmd for DisableHostGroupHostChecks {
    fn to_cmd_string(&self) -> String {
        format!("DISABLE_HOSTGROUP_HOST_CHECKS;{}", self.hostgroup_name)
    }
}

//////////////////////////////////
/// ENABLE_HOST_CHECK

pub struct EnableHostCheck {
    host_name: String,
}

impl EnableHostCheck {
    pub fn new(host_name: String) -> Self {
        Self { host_name }
    }
}

impl NagiosCmd for EnableHostCheck {
    fn to_cmd_string(&self) -> String {
        format!("ENABLE_HOST_CHECK;{}", self.host_name)
    }
}

//////////////////////////////////
/// DISABLE_HOST_CHECK

pub struct DisableHostCheck {
    host_name: String,
}

impl DisableHostCheck {
    pub fn new(host_name: String) -> Self {
        Self { host_name }
    }
}

impl NagiosCmd for DisableHostCheck {
    fn to_cmd_string(&self) -> String {
        format!("DISABLE_HOST_CHECK;{}", self.host_name)
    }
}

//////////////////////////////////
/// ENABLE_HOST_NOTIFICATIONS

pub struct EnableHostNotifications {
    host_name: String,
}

impl EnableHostNotifications {
    pub fn new(host_name: String) -> Self {
        Self { host_name }
    }
}

impl NagiosCmd for EnableHostNotifications {
    fn to_cmd_string(&self) -> String {
        format!("ENABLE_HOST_NOTIFICATIONS;{}", self.host_name)
    }
}

//////////////////////////////////
/// DISABLE_HOST_NOTIFICATIONS

pub struct DisableHostNotifications {
    host_name: String,
}

impl DisableHostNotifications {
    pub fn new(host_name: String) -> Self {
        Self { host_name }
    }
}

impl NagiosCmd for DisableHostNotifications {
    fn to_cmd_string(&self) -> String {
        format!("DISABLE_HOST_NOTIFICATIONS;{}", self.host_name)
    }
}

//////////////////////////////////
/// ENABLE_SVC_CHECK

pub struct EnableSvcCheck {
    host_name: String,
    service_description: String,
}

impl EnableSvcCheck {
    pub fn new(host_name: String, service_description: String) -> Self {
        Self {
            host_name,
            service_description,
        }
    }
}

impl NagiosCmd for EnableSvcCheck {
    fn to_cmd_string(&self) -> String {
        format!(
            "ENABLE_SVC_CHECK;{};{}",
            self.host_name, self.service_description
        )
    }
}

//////////////////////////////////
/// DISABLE_SVC_CHECK

pub struct DisableSvcCheck {
    host_name: String,
    service_description: String,
}

impl DisableSvcCheck {
    pub fn new(host_name: String, service_description: String) -> Self {
        Self {
            host_name,
            service_description,
        }
    }
}

impl NagiosCmd for DisableSvcCheck {
    fn to_cmd_string(&self) -> String {
        format!(
            "DISABLE_SVC_CHECK;{};{}",
            self.host_name, self.service_description
        )
    }
}

//////////////////////////////////
/// ENABLE_SVC_NOTIFICATIONS

pub struct EnableSvcNotifications {
    host_name: String,
    service_description: String,
}

impl EnableSvcNotifications {
    pub fn new(host_name: String, service_description: String) -> Self {
        Self {
            host_name,
            service_description,
        }
    }
}

impl NagiosCmd for EnableSvcNotifications {
    fn to_cmd_string(&self) -> String {
        format!(
            "ENABLE_SVC_NOTIFICATIONS;{};{}",
            self.host_name, self.service_description
        )
    }
}

//////////////////////////////////
/// DISABLE_SVC_NOTIFICATIONS

pub struct DisableSvcNotifications {
    host_name: String,
    service_description: String,
}

impl DisableSvcNotifications {
    pub fn new(host_name: String, service_description: String) -> Self {
        Self {
            host_name,
            service_description,
        }
    }
}

impl NagiosCmd for DisableSvcNotifications {
    fn to_cmd_string(&self) -> String {
        format!(
            "DISABLE_SVC_NOTIFICATIONS;{};{}",
            self.host_name, self.service_description
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::cmd;
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
        let result = cmd::write_cmd_line(&cmd, datetime.timestamp(), &mut buf);

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
                cmd: Box::new(EnableHostGroupHostChecks::new("localhost".to_string())),
                expected: "[1647824400] ENABLE_HOSTGROUP_HOST_CHECKS;localhost\n",
            },
            // DISABLE_HOSTGROUP_HOST_CHECKS
            TestCase {
                cmd: Box::new(DisableHostGroupHostChecks::new("localhost".to_string())),
                expected: "[1647824400] DISABLE_HOSTGROUP_HOST_CHECKS;localhost\n",
            },
            // ENABLE_HOST_CHECK
            TestCase {
                cmd: Box::new(EnableHostCheck::new("localhost".to_string())),
                expected: "[1647824400] ENABLE_HOST_CHECK;localhost\n",
            },
            // DISABLE_HOST_CHECK
            TestCase {
                cmd: Box::new(DisableHostCheck::new("localhost".to_string())),
                expected: "[1647824400] DISABLE_HOST_CHECK;localhost\n",
            },
            // ENABLE_HOST_NOTIFICATIONS
            TestCase {
                cmd: Box::new(EnableHostNotifications::new("localhost".to_string())),
                expected: "[1647824400] ENABLE_HOST_NOTIFICATIONS;localhost\n",
            },
            // DISABLE_HOST_NOTIFICATIONS
            TestCase {
                cmd: Box::new(DisableHostNotifications::new("localhost".to_string())),
                expected: "[1647824400] DISABLE_HOST_NOTIFICATIONS;localhost\n",
            },
            // ENABLE_SVC_CHECK
            TestCase {
                cmd: Box::new(EnableSvcCheck::new(
                    "localhost".to_string(),
                    "Current Load".to_string(),
                )),
                expected: "[1647824400] ENABLE_SVC_CHECK;localhost;Current Load\n",
            },
            // DISABLE_SVC_CHECK
            TestCase {
                cmd: Box::new(DisableSvcCheck::new(
                    "localhost".to_string(),
                    "Current Load".to_string(),
                )),
                expected: "[1647824400] DISABLE_SVC_CHECK;localhost;Current Load\n",
            },
            // ENABLE_SVC_NOTIFICATIONS
            TestCase {
                cmd: Box::new(EnableSvcNotifications::new(
                    "localhost".to_string(),
                    "Current Load".to_string(),
                )),
                expected: "[1647824400] ENABLE_SVC_NOTIFICATIONS;localhost;Current Load\n",
            },
            // DISABLE_SVC_NOTIFICATIONS
            TestCase {
                cmd: Box::new(DisableSvcNotifications::new(
                    "localhost".to_string(),
                    "Current Load".to_string(),
                )),
                expected: "[1647824400] DISABLE_SVC_NOTIFICATIONS;localhost;Current Load\n",
            },
        ];

        for test_case in test_cases {
            assert_eq!(written_string(test_case.cmd), test_case.expected);
        }
    }
}
