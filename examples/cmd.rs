use nagrs::Nagrs;

fn main() {
    let nagrs = Nagrs::new("testdata/nagios.cmd", "testdata/status.dat");
    let cmd = nagrs::nagios::cmd::DisableHostgroupHostChecks {
        hostgroup_name: "localhost".to_string(),
    };
    nagrs.write_cmds(&vec![Box::new(cmd)]).unwrap();
}
