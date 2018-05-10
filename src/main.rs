#[macro_use]
extern crate failure;
extern crate lettre;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate time;
extern crate toml;
#[macro_use]
extern crate clap;

mod args;
mod jobdb;
mod mails;
mod result;

use failure::ResultExt;
use result::Result;
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
    smtp: Cow<'a, str>,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let path = env::current_dir().context("Failed to get current directory")?;

    let arg_matches = args::get_parser().get_matches();
    let cmdline: Vec<_> = arg_matches.values_of("command").unwrap().collect();
    let jobname = arg_matches.value_of("jobname");
    let config_file = {
        let name = arg_matches.value_of("config").unwrap_or("poomas.toml");
        path.join(name)
    };

    let mut file = File::open(config_file).context("Failed to open configuration file")?;
    let mut cfgstr = String::new();
    file.read_to_string(&mut cfgstr)
        .context("Failed to read the configuration file")?;
    let mut config: Config = toml::from_str(&cfgstr).context("Failed to load configuration")?;

    // TODO add some more robust default value handling
    if config.smtp == "" {
        let mut s = config.email.split('@').skip(1);
        let domain = s.next()
            .ok_or(format_err!("Invalid e-mail format: no domain"))?;
        let smtp = "smtp.".to_string() + domain;
        println!("Assuming smtp server: {}", smtp);
        config.smtp = Cow::Owned(smtp);
    }

    let mut cmd = Command::new(&cmdline[0]);
    cmd.args(&cmdline[1..]);
    let cmdline_human = cmdline.join(" ");
    println!("Running: {}", cmdline_human);
    println!("============= Output: =============");

    let start = Instant::now();
    let status = cmd.status().context("Failed to launch the command")?;
    let exec_time = start.elapsed();
    println!("===================================");

    // we want to continue the execution even if this fails
    // I don't currently have an idea how to do it cleaner than to store it
    // looks like a limitation of the failure crate. Is it maintained at all?
    let mut db_save_res = Ok(());
    if let Some(dbfilename) = arg_matches.value_of("database-file") {
        let db = jobdb::JobDB::new(dbfilename.to_owned());
        let res = db.add_job(&cmdline_human, &exec_time);
        match res {
            Ok(_) => eprintln!("Updating the database successful"),
            Err(e) => {
                eprintln!("Error updating the databse: {}", e);
                // This sucks, we want some context, but the types mismatch
                db_save_res = Err(e);
            }
        }
    }

    let email = CommandStatusMail {
        cmdline: cmdline,
        duration: exec_time,
        status: status,
        jobname: jobname,
    }.create_email(&config)
        .context("Failed to build an e-mail")?;
    // FIXME this should return a Result
    send_mail(email, &config);
    eprintln!("E-mail sent!");

    // Hack continued, see the comment before saving the DB ...
    db_save_res
}
