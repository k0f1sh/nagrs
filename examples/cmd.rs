use nagrs::Nagrs;

fn main() {
    let mut nagrs = Nagrs::new("testdata/nagios.cmd", "testdata/status.dat", 10);
    let cmd = nagrs::nagios_cmd::DisableHostGroupHostChecks::new("localhost".to_string());
    nagrs.write_cmds(&vec![Box::new(cmd)]).unwrap();
}
