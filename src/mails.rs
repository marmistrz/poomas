use std::process::ExitStatus;
use std::time::Duration;

use lettre::email::{Email, EmailBuilder};
use lettre::email::error::Error;
use lettre::transport::smtp::{SmtpTransportBuilder, SUBMISSION_PORT};
use lettre::transport::EmailTransport;

use Config;

pub fn send_mail(email: Email, config: &Config) {
    let mut transport = SmtpTransportBuilder::new(("smtp.gmail.com", SUBMISSION_PORT))
        .expect("Failed to create transport")
        .credentials(config.email, config.passwd)
        .build();

    transport.send(email).expect("Failed to send the e-mail");
}

pub struct CommandStatusMail {
    pub cmdline: Vec<String>,
    pub duration: Duration,
    pub status: ExitStatus,
}

impl CommandStatusMail {
    pub fn create_email(&self, config: &Config) -> Result<Email, Error> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::process::ExitStatusExt;

    const CONFIG: Config = Config {
        target: "target",
        email: "email",
        passwd: "passwd",
    };

    #[test]
    fn test_mail_creation() {
        let status = CommandStatusMail {
            cmdline: vec![
                String::from("foo"),
                String::from("bar"),
                String::from("baz"),
            ],
            duration: Duration::from_millis(1024),
            status: ExitStatus::from_raw(0),
        };

        let email = status.create_email(&CONFIG).unwrap();
        let body = format!("{}", email);

        assert!(body.contains("foo bar baz"));
        assert!(body.contains("To: <target>"));
        assert!(body.contains("From: <email>"));
        assert!(body.contains("1.024 s"));
        assert!(body.contains("code: 0"));
    }
}