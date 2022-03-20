use nagrs::nagios::NagiosStatus;

fn main() {
    let path = "testdata/status.dat";
    let status = NagiosStatus::parse_file(path);
    match status {
        Ok(status) => {
            println!("parse ok!");
            println!("{:#?}", status);
        }
        Err(error) => {
            println!("parse error!");
            println!("{:#?}", error);
        }
    }
}
