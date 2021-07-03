use chrono::{DateTime, SecondsFormat, Utc};
use std::error::Error;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entry {
    pub start: DateTime<Utc>,
    pub stop: DateTime<Utc>,
    pub account: String,
    pub description: Option<String>,
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}-{} {}",
            self.start.to_rfc3339_opts(SecondsFormat::Secs, true),
            self.stop.to_rfc3339_opts(SecondsFormat::Secs, true),
            self.account
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RunningEntry {
    pub start: DateTime<Utc>,
    pub account: String,
    pub description: Option<String>,
}

impl fmt::Display for RunningEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}",
            self.start.to_rfc3339_opts(SecondsFormat::Secs, true),
            self.account
        )
    }
}

impl FromStr for RunningEntry {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (start, account) = s.split_once(" ").ok_or(ParseError::MissingStart)?;
        let start = DateTime::from_str(start)?;
        Ok(RunningEntry {
            start,
            account: account.to_string(),
            description: None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    MissingStart,
    DateParseError(chrono::ParseError),
}

impl From<chrono::ParseError> for ParseError {
    fn from(err: chrono::ParseError) -> Self {
        ParseError::DateParseError(err)
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::MissingStart => write!(f, "missing start date"),
            &ParseError::DateParseError(err) => err.fmt(f),
        }
    }
}

impl Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_entry() {
        let entry = Entry {
            start: DateTime::from_str("2021-07-03T10:00:00Z").unwrap(),
            stop: DateTime::from_str("2021-07-03T13:00:00Z").unwrap(),
            account: "Time Tracker".to_string(),
            description: None,
        };

        assert_eq!(
            format!("{}", entry),
            "2021-07-03T10:00:00Z-2021-07-03T13:00:00Z Time Tracker"
        )
    }

    #[test]
    fn display_running_entry() {
        let entry = RunningEntry {
            start: DateTime::from_str("2021-07-03T10:00:00Z").unwrap(),
            account: "Time Tracker".to_string(),
            description: None,
        };

        assert_eq!(format!("{}", entry), "2021-07-03T10:00:00Z Time Tracker");
    }

    #[test]
    fn parse_running_entry() {
        let entry = RunningEntry::from_str("2021-07-03T10:00:00Z Time Tracker").unwrap();

        assert_eq!(
            entry,
            RunningEntry {
                start: DateTime::from_str("2021-07-03T10:00:00Z").unwrap(),
                account: "Time Tracker".to_string(),
                description: None,
            }
        );
    }
}
