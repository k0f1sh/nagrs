# [WIP] A library to get status and write commands for Nagios.

## Usage

example code

```rust
use nagrs::Nagrs;

fn main() {
    let command_file_path = "/usr/local/nagios/var/rw/nagios.cmd";
    let status_file_path = "/usr/local/nagios/var/status.dat";
    let nagrs = Nagrs::new(command_file_path, status_file_path);

    // get nagios status from status.dat
    let nagios_status = nagrs.parse().unwrap();
    println!("{:#?}", nagios_status);

    // write command
    let cmd = nagrs::nagios::cmd::DisableHostgroupHostChecks {
        hostgroup_name: "localhost".to_string(),
    };
    nagrs.write_cmds(&vec![Box::new(cmd)]).unwrap();
}
```
