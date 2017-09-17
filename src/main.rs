extern crate lettre;
#[macro_use]
extern crate serde_derive;
extern crate time;
extern crate toml;

mod mails;

use std::env;
use std::fs::File;
use std::io::Read;
use std::process::{Command, exit};
use std::time::Instant;

use mails::{CommandStatusMail, send_mail};

fn usage() -> ! {
    println!("Usage: {} <command>", env::args().next().unwrap());
    exit(1)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config<'a> {
    target: &'a str,
    email: &'a str,
    passwd: &'a str,
}

fn main() {
    let path = env::current_dir()
        .expect("Failed to get home directory")
        .join("runner.toml");

    let mut file = File::open(path).expect("Failed to open configuration file");
    let mut cfgstr = String::new();
    file.read_to_string(&mut cfgstr).expect(
        "Failed to read the configuration file",
    );
    let config: Config = toml::from_str(&cfgstr).expect("Failed to load configuration");

    let mut args = env::args().skip(1).peekable();
    if args.peek().is_none() {
        usage();
    }
    let cmdline: Vec<_> = args.collect();

    let mut cmd = Command::new(&cmdline[0]);
    cmd.args(&cmdline[1..]);
    println!("Running: {}", cmdline.join(" "));
    println!("============= Output: =============");

    let start = Instant::now();
    let status = cmd.status().unwrap();
    let exec_time = start.elapsed();
    println!("===================================");


    let email = CommandStatusMail {
        cmdline: cmdline,
        duration: exec_time,
        status: status,
    }.create_email(&config)
        .expect("Failed to build an e-mail");
    send_mail(email, &config);
    println!("E-mail sent!");
}
