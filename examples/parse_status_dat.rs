use nagrs::Nagrs;

fn main() {
    let command_file_path = "/usr/local/nagios/var/rw/nagios.cmd";
    let status_file_path = "/usr/local/nagios/var/status.dat";
    let mut nagrs = Nagrs::new(command_file_path, status_file_path, 10);

    let host = nagrs.find_host("localhost").unwrap();
    println!("{:#?}", host);
}
