use chrono::Utc;
use once_cell::sync::Lazy;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;
use tt::{Entry, RunningEntry};

static DEFAULT_RUNNING_FILE: Lazy<String> =
    Lazy::new(|| format!("{}/.tt_running", env::var("HOME").as_deref().unwrap_or(".")));

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, long, parse(from_os_str), env = "TIMETRACKER_FILE")]
    file: PathBuf,

    #[structopt(long, parse(from_os_str), env="TIMETRACKER_RUNNING_FILE", default_value=&DEFAULT_RUNNING_FILE)]
    running_file: PathBuf,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    Export {
        #[structopt(short, long, parse(from_os_str))]
        output: PathBuf,
    },
    Running,
    Start {
        account: String,
    },
    Stop {
        account: Option<String>,
    },
}

fn main() {
    env_logger::init();

    let opt = Opt::from_args();
    log::debug!("{:?}", opt);

    match opt.cmd {
        Command::Export { output } => {
            // Error if there's already a file located at the output path
            if output.exists() {
                panic!("there is already a file at the output path");
            }

            // Read every entry and format as a timeclock entry
            let file = OpenOptions::new()
                .read(true)
                .open(&opt.file)
                .expect("could not open file");
            let timeclock = BufReader::new(file)
                .lines()
                .map(|line| line.unwrap())
                .map(|line| Entry::from_str(&line).unwrap())
                .map(|entry| entry.format_as_timeclock())
                .collect::<Vec<String>>()
                .join("\n");

            // Write the timeclock formatted entries to the output file
            fs::write(output, timeclock).expect("could not write to output file");
        }

        Command::Running => {
            // Open the file with running entries
            let running_file = OpenOptions::new()
                .read(true)
                .open(&opt.running_file)
                .expect("could not open running file");

            // Print each running entry
            BufReader::new(running_file)
                .lines()
                .map(|line| line.unwrap())
                .map(|line| RunningEntry::from_str(&line).unwrap())
                .for_each(|entry| println!("{}", entry));
        }

        Command::Start { account } => {
            // Create the new running entry
            let running_entry = RunningEntry {
                start: Utc::now(),
                account: account.clone(),
                description: None,
            };

            // Error if there is already a running entry for the account
            if opt.running_file.exists() {
                let running_file = OpenOptions::new()
                    .read(true)
                    .open(&opt.running_file)
                    .expect("could not open running file");
                if BufReader::new(running_file)
                    .lines()
                    .map(|line| line.unwrap())
                    .map(|line| RunningEntry::from_str(&line).unwrap())
                    .any(|entry| entry.account == account)
                {
                    panic!(
                        r#"there is already a running entry for the account "{}""#,
                        account
                    );
                }
            }

            // Open the file for running entries and append the new entry at the end
            let mut running_file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&opt.running_file)
                .expect("could not open running file");
            writeln!(running_file, "{}", running_entry).expect("could not write to running file");
        }

        Command::Stop { account } => {
            let running_file = OpenOptions::new()
                .read(true)
                .open(&opt.running_file)
                .expect("could not open running file");

            let mut running_entries: Vec<RunningEntry> = BufReader::new(running_file)
                .lines()
                .map(|line| line.unwrap())
                .map(|line| RunningEntry::from_str(&line).unwrap())
                .collect();

            // Error immediately if there are no running entries
            if running_entries.is_empty() {
                panic!("no running entries");
            }

            let position = match account {
                Some(account) => running_entries
                    .iter()
                    .position(|entry| entry.account == account)
                    .unwrap_or_else(|| {
                        panic!(
                            r#"no running entries for the account "{}" were found"#,
                            account
                        )
                    }),
                None => {
                    if running_entries.len() != 1 {
                        panic!(
                            "account must be specified when there is more than one running entry"
                        );
                    }
                    0
                }
            };

            // Extract the running entry and remove it from the collection
            let running_entry = running_entries.remove(position);

            // Create a new complete entry
            let entry = Entry {
                start: running_entry.start,
                stop: Utc::now(),
                account: running_entry.account,
                description: running_entry.description,
            };

            // Write the new entry
            let mut entry_file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(opt.file)
                .expect("could not open entries file");
            writeln!(entry_file, "{}", entry).expect("could not write to entries file");

            // Write the remaining running entries to the running file
            fs::write(
                &opt.running_file,
                running_entries
                    .iter()
                    .map(|entry| format!("{}", entry))
                    .collect::<Vec<String>>()
                    .join("\n"),
            )
            .expect("could not write to running file");
        }
    }
}
