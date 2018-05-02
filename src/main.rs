extern crate lettre;
#[macro_use]
extern crate serde_derive;
extern crate time;
extern crate toml;
#[macro_use]
extern crate clap;

mod args;
mod mails;

use std::borrow::Cow;
use std::env;
use std::fs::File;
use std::io::Read;
use std::process::Command;
use std::time::Instant;

use mails::{send_mail, CommandStatusMail};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config<'a> {
    target: &'a str,
    email: &'a str,
    passwd: &'a str,
    #[serde(default)]
    smtp: Cow<'a, str>, // FIXME use a Cow
}

fn main() {
    let path = env::current_dir().expect("Failed to get current directory");

    let arg_matches = args::get_parser().get_matches();
    let cmdline: Vec<_> = arg_matches.values_of("command").unwrap().collect();
    let jobname = arg_matches.value_of("jobname");
    let config_file = {
        let name = arg_matches.value_of("config").unwrap_or("poomas.toml");
        path.join(name)
    };

    let mut file = File::open(config_file).expect("Failed to open configuration file");
    let mut cfgstr = String::new();
    file.read_to_string(&mut cfgstr)
        .expect("Failed to read the configuration file");
    let mut config: Config = toml::from_str(&cfgstr).expect("Failed to load configuration");

    // TODO add some more robust default value handling
    if config.smtp == "" {
        let mut s = config.email.split('@').skip(1);
        let domain = s.next().expect("Invalid e-mail format: no domain");
        let smtp = "smtp.".to_string() + domain;
        println!("Assuming smtp server: {}", smtp);
        config.smtp = Cow::Owned(smtp);
    }

    let mut cmd = Command::new(&cmdline[0]);
    cmd.args(&cmdline[1..]);
    println!("Running: {}", cmdline.join(" "));
    println!("============= Output: =============");

    let start = Instant::now();
    let status = cmd.status().expect("Failed to launch the command");
    let exec_time = start.elapsed();
    println!("===================================");

    let email = CommandStatusMail {
        cmdline: cmdline,
        duration: exec_time,
        status: status,
        jobname: jobname,
    }.create_email(&config)
        .expect("Failed to build an e-mail");
    send_mail(email, &config);
    println!("E-mail sent!");
}
