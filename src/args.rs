use clap::{App, Arg};

const APP: &'static str = env!("CARGO_PKG_NAME");

pub fn get_parser<'a, 'b>() -> App<'a, 'b> {
    App::new(APP)
        .version(crate_version!())
        .author(crate_authors!())
        .about("Poor man's slurm")
        .arg(
            Arg::with_name("jobname")
                .short("J")
                .required(false)
                .takes_value(true)
                .help("Set the job name"),
        )
        .arg(
            Arg::with_name("config")
                .short("c")
                .required(false)
                .takes_value(true)
                .help("Override the configuration file. Defaults to poomas.toml"),
        )
        .arg(
            Arg::with_name("command")
                .multiple(true)
                .required(true)
                .help("The command to be executed"),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_only_command() {
        let matches =
            get_parser().get_matches_from(&["./executable", "grep", "bash", "/etc/passwd"]);

        assert_eq!(
            matches.values_of("command").unwrap().collect::<Vec<_>>(),
            ["grep", "bash", "/etc/passwd"]
        );
    }

    #[test]
    fn test_jobname() {
        let matches = get_parser().get_matches_from(&["./executable", "-J", "xd", "echo"]);

        assert_eq!(
            matches.values_of("command").unwrap().collect::<Vec<_>>(),
            ["echo"]
        );
        assert_eq!(matches.value_of("jobname").unwrap(), "xd");
    }

    #[test]
    fn test_config() {
        let matches =
            get_parser().get_matches_from(&["./executable", "-J", "xd", "-c", "xd.toml", "echo"]);

        assert_eq!(
            matches.values_of("config").unwrap().collect::<Vec<_>>(),
            ["xd.toml"]
        );
        assert_eq!(matches.value_of("jobname").unwrap(), "xd");
    }

    /*#[test]
    #[should_panic]
    fn test_double_jobname() {
        get_parser().get_matches_from(&["./executable", "-J", "xd", "-J", "xdddd", "echo"]);
    }*/
}
