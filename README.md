# [WIP] A library for managing Nagios.

## Usage

```
use nagrs::Nagrs;

fn main() {
    let command_file_path = "/usr/local/nagios/var/rw/nagios.cmd";
    let status_file_path = "/usr/local/nagios/var/status.dat";
    let mut nagrs = Nagrs::new(command_file_path, status_file_path, 10);

    let host = nagrs.find_host("localhost").unwrap();
    println!("{:#?}", host);
}
```

result:
```
Some(
    Host {
        host_name: "localhost",
        modified_attributes: ModifiedAttributes(
            0,
        ),
        check_command: "check-host-alive",
        check_period: "24x7",
        notification_period: "workhours",
        importance: 0,
        check_interval: 5.0,
        retry_interval: 1.0,
        event_handler: "",
        has_been_checked: true,
        should_be_scheduled: true,
        check_execution_time: 4.196,
        check_latency: 0.368,
        check_type: Active,
        current_state: Up,
        last_hard_state: Up,
        plugin_output: "PING OK - Packet loss = 0%, RTA = 0.04 ms",
        long_plugin_output: "",
        performance_data: "rta=0.041000ms;3000.000000;5000.000000;0.000000 pl=0%;80;100;0",
        last_check: Some(
            2022-03-20T11:22:58Z,
        ),
        next_check: Some(
            2022-03-20T11:27:58Z,
        ),
        check_options: CheckOptions(
            0,
        ),
        current_attempt: 1,
        max_attempts: 10,
        state_type: Hard,
        last_state_change: None,
        last_hard_state_change: None,
        last_time_up: Some(
            2022-03-20T11:22:58Z,
        ),
        last_time_down: None,
        last_time_unreachable: None,
        last_notification: None,
        next_notification: None,
        no_more_notifications: false,
        current_notification_number: 0,
        notifications_enabled: true,
        problem_has_been_acknowledged: false,
        acknowledgement_type: None,
        active_checks_enabled: true,
        passive_checks_enabled: true,
        event_handler_enabled: true,
        flap_detection_enabled: true,
        process_performance_data: true,
        obsess: true,
        last_update: Some(
            2022-03-20T11:23:57Z,
        ),
        is_flapping: false,
        percent_state_change: 0.0,
        scheduled_downtime_depth: 0,
    },
)
```