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

    let services = nagrs.find_services("localhost");
    match &services {
        Ok(services) => {
            println!("ok!");
            println!("{:#?}", services);
        }
        Err(error) => {
            println!("error!");
            println!("{:#?}", error);
        }
    };
}
