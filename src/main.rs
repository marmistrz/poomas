extern crate lettre;
#[macro_use]
extern crate serde_derive;
extern crate time;
extern crate toml;

use std::env;
use std::fs::File;
use std::io::Read;
use std::process::{Command, exit, ExitStatus};
use std::time::{Duration, Instant};

use lettre::email::{Email, EmailBuilder};
use lettre::email::error::Error;
use lettre::transport::smtp::{SmtpTransportBuilder, SUBMISSION_PORT};
use lettre::transport::EmailTransport;

fn usage() -> ! {
    println!("Usage: {} <command>", env::args().next().unwrap());
    exit(1)
}

#[derive(Debug, Serialize, Deserialize)]
struct Config<'a> {
    target: &'a str,
    email: &'a str,
    passwd: &'a str,
}

fn send_mail(email: Email, config: &Config) {
    let mut transport = SmtpTransportBuilder::new(("smtp.gmail.com", SUBMISSION_PORT))
        .expect("Failed to create transport")
        .credentials(config.email, config.passwd)
        .build();

    transport.send(email).expect("Failed to send the e-mail");
}

struct CommandStatusMail {
    cmdline: Vec<String>,
    duration: Duration,
    status: ExitStatus,
}

impl CommandStatusMail {
    fn create_email(&self, config: &Config) -> Result<Email, Error> {
        let body = format!(
            "Command: {}\n\
            Execution time: {}.{:03} s\n\
            Exit code: {}",
            self.cmdline.join(" "),
            self.duration.as_secs(),
            self.duration.subsec_nanos() / 1e6 as u32,
            match self.status.code() {
                Some(code) => code.to_string(),
                None => String::from("killed by a signal"),
            }
        );

        EmailBuilder::new()
            .to(config.target)
            .from(config.email)
            .subject("Computation finished")
            .text(&body)
            .build()
    }
}

fn main() {
    let path = env::home_dir()
        .expect("Failed to get home directory")
        .join(".config")
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
