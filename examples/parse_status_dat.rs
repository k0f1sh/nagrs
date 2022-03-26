use nagrs::{nagios::NagiosStatus, Nagrs};

fn main() {
    let mut nagrs = Nagrs::new("", "testdata/status.dat");
    nagrs.load().unwrap();

    let host = nagrs.find_host("localhost");
    match &host {
        Ok(host) => {
            println!("ok!");
            println!("{:#?}", host);
        }
        Err(error) => {
            println!("error!");
            println!("{:#?}", error);
        }
    };
}
