use failure::ResultExt;
use result::Result;
use std::fs;
use std::time::Duration;

use toml;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct Job {
    id: i32,
    command: String,
    time: u64,
}

#[derive(Serialize, Deserialize)]
struct Jobs {
    #[serde(default)]
    db: Vec<Job>,
}

impl Jobs {
    pub fn push(&mut self, x: Job) {
        self.db.push(x)
    }
}

pub struct JobDB {
    file: String,
}

impl JobDB {
    pub fn new(file: String) -> Self {
        JobDB { file: file }
    }

    pub fn add_job(&self, command: &str, duration: &Duration) -> Result<()> {
        let job = Job {
            id: 0,
            command: command.to_owned(),
            time: duration.as_secs(),
        };
        update_jobs(job, &self.file)
    }
}

fn update_jobs(job: Job, file: &str) -> Result<()> {
    let db_raw = fs::read_to_string(file).unwrap_or("".to_owned());
    let db_raw = update_jobs_str(job, db_raw)?;
    fs::write(file, db_raw).context("Could not write the new DB")?;
    Ok(())
}

fn update_jobs_str(mut job: Job, db_raw: String) -> Result<String> {
    let mut db: Jobs = toml::from_str(&db_raw).context("Could not parse the DB")?;
    let maxid = db.db.iter().map(|j| j.id + 1).max().unwrap_or(0);
    job.id = maxid;
    db.push(job);
    Ok(toml::to_string_pretty(&db).expect("New DB malformed"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_jobs() {
        let job = Job {
            id: 0,
            command: "".to_owned(),
            time: 0,
        };
        let output = update_jobs_str(job.clone(), "".to_owned()).unwrap();
        let db: Jobs = toml::from_str(&output).unwrap();
        assert!(db.db == vec![job.clone()]);

        let job2 = Job {
            id: 1,
            command: "xd".to_owned(),
            time: 200,
        };

        let output = update_jobs_str(job2.clone(), output).unwrap();
        let db: Jobs = toml::from_str(&output).unwrap();
        assert!(db.db == vec![job, job2]);
    }
}
