use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

////////////////////////////////////
// filed types

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CheckType {
    Active,  // 0
    Passive, // 1
    Parent,  // 2
    File,    // 3
    Other,   // 4
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// current_state (host)
/// last_hard_state (host)
pub enum HostState {
    Up,          // 0
    Down,        // 1
    Unreachable, // 2
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AcknowledgementType {
    None,   // 0
    Normal, // 1
    Sticky, // 2
}

////////////////////////////////////
// error

#[derive(Error, Debug, PartialEq)]
pub enum ConvertError {
    #[error("key does not exists: {0}")]
    KeyDoesNotExists(String),
    #[error("failed to parse: {0} -> {1}")]
    FailedToParse(String, String),
    #[error("invalid boolean value: {0}")]
    InvalidBooleanValue(String),
    #[error("invalid host state value: {0}")]
    InvalidHostStateValue(String),
    #[error("invalid check type value: {0}")]
    InvalidCheckTypeValue(String),
    #[error("invalid acknowledgement type value: {0}")]
    InvalidAcknowledgementTypeValue(String),
}

////////////////////////////////////
// nagios status

fn get_raw<'a>(
    key: &str,
    key_values: &'a HashMap<String, String>,
) -> std::result::Result<&'a String, ConvertError> {
    key_values
        .get(key)
        .ok_or(ConvertError::KeyDoesNotExists(key.into()))
}

fn get_bool(
    key: &str,
    key_values: &HashMap<String, String>,
) -> std::result::Result<bool, ConvertError> {
    match get_raw(key, key_values)?.as_str() {
        "0" => Ok(false),
        "1" => Ok(true),
        s => Err(ConvertError::InvalidBooleanValue(s.into())),
    }
}

fn get_string(
    key: &str,
    key_values: &HashMap<String, String>,
) -> std::result::Result<String, ConvertError> {
    let s = get_raw(key, key_values)?;
    Ok(s.into())
}

fn get_u32(
    key: &str,
    key_values: &HashMap<String, String>,
) -> std::result::Result<u32, ConvertError> {
    let s = get_raw(key, key_values)?;
    s.parse::<u32>()
        .map_err(|_| ConvertError::FailedToParse(s.to_string(), "u32".to_string()))
}

fn get_f64(
    key: &str,
    key_values: &HashMap<String, String>,
) -> std::result::Result<f64, ConvertError> {
    let s = get_raw(key, key_values)?;
    s.parse::<f64>()
        .map_err(|_| ConvertError::FailedToParse(s.to_string(), "f64".to_string()))
}

fn get_datetime(
    key: &str,
    key_values: &HashMap<String, String>,
) -> std::result::Result<DateTime<Utc>, ConvertError> {
    let s = get_raw(key, key_values)?;
    let timestamp = s
        .parse::<i64>()
        .map_err(|_| ConvertError::FailedToParse(s.to_string(), "DateTime<Utc>".to_string()))?;
    Ok(DateTime::<Utc>::from_utc(
        NaiveDateTime::from_timestamp(timestamp, 0),
        Utc,
    ))
}

fn get_check_type(
    key: &str,
    key_values: &HashMap<String, String>,
) -> std::result::Result<CheckType, ConvertError> {
    match get_raw(key, key_values)?.as_str() {
        "0" => Ok(CheckType::Active),
        "1" => Ok(CheckType::Passive),
        "2" => Ok(CheckType::Parent),
        "3" => Ok(CheckType::File),
        "4" => Ok(CheckType::Other),
        s => Err(ConvertError::InvalidCheckTypeValue(s.into())),
    }
}

fn get_host_state(
    key: &str,
    key_values: &HashMap<String, String>,
) -> std::result::Result<HostState, ConvertError> {
    match get_raw(key, key_values)?.as_str() {
        "0" => Ok(HostState::Up),
        "1" => Ok(HostState::Down),
        "2" => Ok(HostState::Unreachable),
        s => Err(ConvertError::InvalidHostStateValue(s.into())),
    }
}

fn get_acknowledgement_type(
    key: &str,
    key_values: &HashMap<String, String>,
) -> std::result::Result<AcknowledgementType, ConvertError> {
    match get_raw(key, key_values)?.as_str() {
        "0" => Ok(AcknowledgementType::None),
        "1" => Ok(AcknowledgementType::Normal),
        "2" => Ok(AcknowledgementType::Sticky),
        s => Err(ConvertError::InvalidAcknowledgementTypeValue(s.into())),
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Host {
    pub host_name: String,
    pub check_command: String,
    pub check_period: String,
    pub notification_period: String,
    pub importance: u32,
    pub check_interval: f64,
    pub retry_interval: f64,
    pub event_handler: String,
    pub has_been_checked: bool,
    pub should_be_scheduled: bool,
    pub check_execution_time: f64,
    pub check_latency: f64,
    pub check_type: CheckType,
    pub current_state: HostState,
    pub last_hard_state: HostState,
    pub plugin_output: String,
    pub long_plugin_output: String,
    pub performance_data: String,
    pub last_hard_state_change: DateTime<Utc>,
    pub last_time_up: DateTime<Utc>,
    pub last_time_down: DateTime<Utc>,
    pub last_time_unreachable: DateTime<Utc>,
    pub last_notification: DateTime<Utc>,
    pub next_notification: DateTime<Utc>,
    pub no_more_notifications: bool,
    pub current_notification_number: u32,
    pub notifications_enabled: bool,
    pub problem_has_been_acknowledged: bool,
    pub acknowledgement_type: AcknowledgementType,
    pub active_checks_enabled: bool,
    pub passive_checks_enabled: bool,
    pub event_handler_enabled: bool,
    pub flap_detection_enabled: bool,
    pub process_performance_data: bool,
    pub obsess: bool,
    pub last_update: DateTime<Utc>,
    pub is_flapping: bool,
    pub percent_state_change: f64,
    pub scheduled_downtime_depth: u32,
    // TODO modified_attributes
    // TODO *_id
    // TODO custom variables
}

impl TryFrom<HashMap<String, String>> for Host {
    type Error = ConvertError;

    fn try_from(key_values: HashMap<String, String>) -> std::result::Result<Self, Self::Error> {
        Ok(Host {
            host_name: get_string("host_name", &key_values)?,
            check_command: get_string("check_command", &key_values)?,
            check_period: get_string("check_period", &key_values)?,
            notification_period: get_string("notification_period", &key_values)?,
            importance: get_u32("importance", &key_values)?,
            check_interval: get_f64("check_interval", &key_values)?,
            retry_interval: get_f64("retry_interval", &key_values)?,
            event_handler: get_string("event_handler", &key_values)?,
            has_been_checked: get_bool("has_been_checked", &key_values)?,
            should_be_scheduled: get_bool("should_be_scheduled", &key_values)?,
            check_execution_time: get_f64("check_execution_time", &key_values)?,
            check_latency: get_f64("check_latency", &key_values)?,
            check_type: get_check_type("check_type", &key_values)?,
            current_state: get_host_state("current_state", &key_values)?,
            last_hard_state: get_host_state("last_hard_state", &key_values)?,
            plugin_output: get_string("plugin_output", &key_values)?,
            long_plugin_output: get_string("long_plugin_output", &key_values)?,
            performance_data: get_string("performance_data", &key_values)?,
            last_hard_state_change: get_datetime("last_hard_state_change", &key_values)?,
            last_time_up: get_datetime("last_time_up", &key_values)?,
            last_time_down: get_datetime("last_time_down", &key_values)?,
            last_time_unreachable: get_datetime("last_time_unreachable", &key_values)?,
            last_notification: get_datetime("last_notification", &key_values)?,
            next_notification: get_datetime("next_notification", &key_values)?,
            no_more_notifications: get_bool("no_more_notifications", &key_values)?,
            current_notification_number: get_u32("current_notification_number", &key_values)?,
            notifications_enabled: get_bool("notifications_enabled", &key_values)?,
            problem_has_been_acknowledged: get_bool("problem_has_been_acknowledged", &key_values)?,
            acknowledgement_type: get_acknowledgement_type("acknowledgement_type", &key_values)?,
            active_checks_enabled: get_bool("active_checks_enabled", &key_values)?,
            passive_checks_enabled: get_bool("passive_checks_enabled", &key_values)?,
            event_handler_enabled: get_bool("event_handler_enabled", &key_values)?,
            flap_detection_enabled: get_bool("flap_detection_enabled", &key_values)?,
            process_performance_data: get_bool("process_performance_data", &key_values)?,
            obsess: get_bool("obsess", &key_values)?,
            last_update: get_datetime("last_update", &key_values)?,
            is_flapping: get_bool("is_flapping", &key_values)?,
            percent_state_change: get_f64("percent_state_change", &key_values)?,
            scheduled_downtime_depth: get_u32("scheduled_downtime_depth", &key_values)?,
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Service {
    pub host_name: String,
    pub check_command: String,
    pub service_description: String,
    pub active_checks_enabled: bool,
    pub passive_checks_enabled: bool,
    pub obsess: bool,
    pub event_handler_enabled: bool,
    pub flap_detection_enabled: bool,
    pub notifications_enabled: bool,
    // TODO add fields as needed
}

impl TryFrom<HashMap<String, String>> for Service {
    type Error = ConvertError;

    fn try_from(key_values: HashMap<String, String>) -> std::result::Result<Self, Self::Error> {
        let host_name = key_values
            .get("host_name")
            .ok_or(ConvertError::KeyDoesNotExists("host_name".into()))
            .map(|h| h.into())?;
        let service_description = key_values
            .get("service_description")
            .ok_or(ConvertError::KeyDoesNotExists("service_description".into()))?;

        let check_command = key_values
            .get("check_command")
            .ok_or(ConvertError::KeyDoesNotExists("check_command".into()))?;

        let active_checks_enabled = get_bool("active_checks_enabled", &key_values)?;
        let passive_checks_enabled = get_bool("passive_checks_enabled", &key_values)?;
        let obsess = get_bool("obsess", &key_values)?;
        let event_handler_enabled = get_bool("event_handler_enabled", &key_values)?;
        let flap_detection_enabled = get_bool("flap_detection_enabled", &key_values)?;
        let notifications_enabled = get_bool("notifications_enabled", &key_values)?;

        Ok(Service {
            host_name: host_name,
            service_description: service_description.to_owned(),
            check_command: check_command.to_owned(),
            active_checks_enabled,
            passive_checks_enabled,
            obsess,
            event_handler_enabled,
            flap_detection_enabled,
            notifications_enabled,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_get_raw() {
        let key_values: HashMap<String, String> = HashMap::from([("key".into(), "value".into())]);
        assert_eq!(get_raw("key", &key_values).unwrap(), "value");
    }

    #[test]
    fn test_get_bool() {
        struct TestCase<'a>(&'a str, Result<bool, ConvertError>);
        let test_cases = vec![
            TestCase("0", Ok(false)),
            TestCase("1", Ok(true)),
            TestCase("2", Err(ConvertError::InvalidBooleanValue("2".into()))),
            TestCase(
                "hoge",
                Err(ConvertError::InvalidBooleanValue("hoge".into())),
            ),
        ];

        for test_case in test_cases {
            let key_values: HashMap<String, String> =
                HashMap::from([("key".into(), test_case.0.into())]);
            assert_eq!(get_bool("key", &key_values), test_case.1);
        }
    }

    #[test]
    fn test_get_string() {
        struct TestCase<'a>(&'a str, Result<String, ConvertError>);
        let test_cases = vec![
            TestCase("hoge", Ok("hoge".into())),
            TestCase("fuga", Ok("fuga".into())),
        ];

        for test_case in test_cases {
            let key_values: HashMap<String, String> =
                HashMap::from([("key".into(), test_case.0.into())]);
            assert_eq!(get_string("key", &key_values), test_case.1);
        }
    }

    #[test]
    fn test_get_u32() {
        struct TestCase<'a>(&'a str, Result<u32, ConvertError>);
        let test_cases = vec![
            TestCase("0", Ok(0)),
            TestCase("1", Ok(1)),
            TestCase(
                "hoge",
                Err(ConvertError::FailedToParse("hoge".into(), "u32".into())),
            ),
        ];
        for test_case in test_cases {
            let key_values: HashMap<String, String> =
                HashMap::from([("key".into(), test_case.0.into())]);
            assert_eq!(get_u32("key", &key_values), test_case.1);
        }
    }

    #[test]
    fn test_get_f64() {
        struct TestCase<'a>(&'a str, Result<f64, ConvertError>);
        let test_cases = vec![
            TestCase("1.000000", Ok(1.0)),
            TestCase("5.000000", Ok(5.0)),
            TestCase(
                "hoge",
                Err(ConvertError::FailedToParse("hoge".into(), "f64".into())),
            ),
        ];
        for test_case in test_cases {
            let key_values: HashMap<String, String> =
                HashMap::from([("key".into(), test_case.0.into())]);
            assert_eq!(get_f64("key", &key_values), test_case.1);
        }
    }

    #[test]
    fn test_get_datetime() {
        struct TestCase<'a>(&'a str, Result<DateTime<Utc>, ConvertError>);
        let test_cases = vec![
            TestCase(
                "1647775378",
                Ok(chrono::Utc.ymd(2022, 3, 20).and_hms(11, 22, 58)),
            ),
            TestCase(
                "hoge",
                Err(ConvertError::FailedToParse(
                    "hoge".into(),
                    "DateTime<Utc>".into(),
                )),
            ),
        ];
        for test_case in test_cases {
            let key_values: HashMap<String, String> =
                HashMap::from([("key".into(), test_case.0.into())]);
            assert_eq!(get_datetime("key", &key_values), test_case.1);
        }
    }

    #[test]
    fn test_get_check_type() {
        struct TestCase<'a>(&'a str, Result<CheckType, ConvertError>);
        let test_cases = vec![
            TestCase("0", Ok(CheckType::Active)),
            TestCase("1", Ok(CheckType::Passive)),
            TestCase("2", Ok(CheckType::Parent)),
            TestCase("3", Ok(CheckType::File)),
            TestCase("4", Ok(CheckType::Other)),
            TestCase(
                "hoge",
                Err(ConvertError::InvalidCheckTypeValue("hoge".into())),
            ),
        ];
        for test_case in test_cases {
            let key_values: HashMap<String, String> =
                HashMap::from([("key".into(), test_case.0.into())]);
            assert_eq!(get_check_type("key", &key_values), test_case.1);
        }
    }

    #[test]
    fn test_get_host_state() {
        struct TestCase<'a>(&'a str, Result<HostState, ConvertError>);
        let test_cases = vec![
            TestCase("0", Ok(HostState::Up)),
            TestCase("1", Ok(HostState::Down)),
            TestCase("2", Ok(HostState::Unreachable)),
            TestCase(
                "hoge",
                Err(ConvertError::InvalidHostStateValue("hoge".into())),
            ),
        ];
        for test_case in test_cases {
            let key_values: HashMap<String, String> =
                HashMap::from([("key".into(), test_case.0.into())]);
            assert_eq!(get_host_state("key", &key_values), test_case.1);
        }
    }

    #[test]
    fn test_get_acknowledgement_type() {
        struct TestCase<'a>(&'a str, Result<AcknowledgementType, ConvertError>);
        let test_cases = vec![
            TestCase("0", Ok(AcknowledgementType::None)),
            TestCase("1", Ok(AcknowledgementType::Normal)),
            TestCase("2", Ok(AcknowledgementType::Sticky)),
            TestCase(
                "hoge",
                Err(ConvertError::InvalidAcknowledgementTypeValue("hoge".into())),
            ),
        ];
        for test_case in test_cases {
            let key_values: HashMap<String, String> =
                HashMap::from([("key".into(), test_case.0.into())]);
            assert_eq!(get_acknowledgement_type("key", &key_values), test_case.1);
        }
    }

    #[test]
    fn host_try_from() {
        let key_values = HashMap::from([
            ("host_name".into(), "localhost".into()),
            ("check_command".into(), "check-host-alive".into()),
            ("check_period".into(), "24x7".into()),
            ("notification_period".into(), "workhours".into()),
            ("importance".into(), "0".into()),
            ("check_interval".into(), "5.000000".into()),
            ("retry_interval".into(), "1.000000".into()),
            ("event_handler".into(), "".into()),
            ("has_been_checked".into(), "1".into()),
            ("should_be_scheduled".into(), "1".into()),
            ("check_execution_time".into(), "4.196".into()),
            ("check_latency".into(), "0.368".into()),
            ("check_type".into(), "0".into()),
            ("current_state".into(), "0".into()),
            ("last_hard_state".into(), "0".into()),
            ("last_event_id".into(), "0".into()),
            (
                "plugin_output".into(),
                "PING OK - Packet loss = 0%, RTA = 0.04 ms".into(),
            ),
            ("long_plugin_output".into(), "".into()),
            (
                "performance_data".into(),
                "rta=0.041000ms;3000.000000;5000.000000;0.000000 pl=0%;80;100;0".into(),
            ),
            ("last_check".into(), "1647775378".into()),
            ("next_check".into(), "1647775678".into()),
            ("check_options".into(), "0".into()),
            ("current_attempt".into(), "1".into()),
            ("max_attempts".into(), "10".into()),
            ("state_type".into(), "1".into()),
            ("last_state_change".into(), "0".into()),
            ("last_hard_state_change".into(), "0".into()),
            ("last_time_up".into(), "1647775378".into()),
            ("last_time_down".into(), "0".into()),
            ("last_time_unreachable".into(), "0".into()),
            ("last_notification".into(), "0".into()),
            ("next_notification".into(), "0".into()),
            ("no_more_notifications".into(), "0".into()),
            ("current_notification_number".into(), "0".into()),
            ("notifications_enabled".into(), "1".into()),
            ("problem_has_been_acknowledged".into(), "0".into()),
            ("acknowledgement_type".into(), "0".into()),
            ("active_checks_enabled".into(), "1".into()),
            ("passive_checks_enabled".into(), "1".into()),
            ("event_handler_enabled".into(), "1".into()),
            ("flap_detection_enabled".into(), "1".into()),
            ("process_performance_data".into(), "1".into()),
            ("obsess".into(), "1".into()),
            ("last_update".into(), "1647775437".into()),
            ("is_flapping".into(), "0".into()),
            ("percent_state_change".into(), "0.00".into()),
            ("scheduled_downtime_depth".into(), "0".into()),
        ]);

        let host = Host::try_from(key_values);
        assert_eq!(host.is_err(), false);

        let host = host.unwrap();
        assert_eq!(host.host_name, "localhost".to_string());
        assert_eq!(host.active_checks_enabled, true);
        assert_eq!(host.passive_checks_enabled, true);
        assert_eq!(host.obsess, true);
        assert_eq!(host.event_handler_enabled, true);
        assert_eq!(host.flap_detection_enabled, true);
        assert_eq!(host.notifications_enabled, true);
    }
}
