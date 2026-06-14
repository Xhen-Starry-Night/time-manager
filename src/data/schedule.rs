use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::data::models::{RecurrenceRule, Schedule};
use crate::data::{DataError, Result};

pub fn parse_ics(content: &str) -> Result<Schedule> {
    let lines: Vec<&str> = content.lines().collect();

    let id = lines
        .iter()
        .find(|l| l.starts_with("UID:"))
        .and_then(|l| l.strip_prefix("UID:"))
        .and_then(|s| s.split('@').next())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| DataError::InvalidData("Missing UID".into()))?;

    let summary = lines
        .iter()
        .find(|l| l.starts_with("SUMMARY:"))
        .and_then(|l| l.strip_prefix("SUMMARY:"))
        .unwrap_or("")
        .to_string();

    let dtstart = lines
        .iter()
        .find(|l| l.starts_with("DTSTART:"))
        .and_then(|l| l.strip_prefix("DTSTART:"))
        .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y%m%dT%H%M%SZ").ok())
        .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc))
        .ok_or_else(|| DataError::InvalidData("Missing DTSTART".into()))?;

    let dtend = lines
        .iter()
        .find(|l| l.starts_with("DTEND:"))
        .and_then(|l| l.strip_prefix("DTEND:"))
        .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y%m%dT%H%M%SZ").ok())
        .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc))
        .ok_or_else(|| DataError::InvalidData("Missing DTEND".into()))?;

    let description = lines
        .iter()
        .find(|l| l.starts_with("DESCRIPTION:"))
        .and_then(|l| l.strip_prefix("DESCRIPTION:"))
        .map(String::from);

    let location = lines
        .iter()
        .find(|l| l.starts_with("LOCATION:"))
        .and_then(|l| l.strip_prefix("LOCATION:"))
        .map(String::from);

    let categories = lines
        .iter()
        .find(|l| l.starts_with("CATEGORIES:"))
        .and_then(|l| l.strip_prefix("CATEGORIES:"))
        .map(|s| s.split(',').map(String::from).collect())
        .unwrap_or_default();

    let priority = lines
        .iter()
        .find(|l| l.starts_with("PRIORITY:"))
        .and_then(|l| l.strip_prefix("PRIORITY:"))
        .and_then(|s| s.parse::<i32>().ok());

    let rrule = lines
        .iter()
        .find(|l| l.starts_with("RRULE:"))
        .and_then(|l| l.strip_prefix("RRULE:"))
        .map(RecurrenceRule::from_rrule)
        .unwrap_or(RecurrenceRule::None);

    let reminder_minutes = lines
        .iter()
        .skip_while(|l| !l.starts_with("BEGIN:VALARM"))
        .skip(1)
        .take_while(|l| !l.starts_with("END:VALARM"))
        .find(|l| l.starts_with("TRIGGER:-PT"))
        .and_then(|l| l.strip_prefix("TRIGGER:-PT"))
        .and_then(|s| s.strip_suffix('M'))
        .and_then(|s| s.parse::<i32>().ok());

    Ok(Schedule {
        id,
        summary,
        dtstart,
        dtend,
        description,
        location,
        categories,
        priority,
        rrule,
        reminder_minutes,
    })
}

pub fn format_ics(schedule: &Schedule) -> String {
    let mut ics = format!(
        "BEGIN:VCALENDAR\nVERSION:2.0\nPRODID:-//Time Manager//EN\nCALSCALE:GREGORIAN\nBEGIN:VEVENT\nUID:{}@time-manager\nDTSTART:{}\nDTEND:{}\nSUMMARY:{}\nCREATED:{}\nDTSTAMP:{}\n",
        schedule.id,
        schedule.dtstart.format("%Y%m%dT%H%M%SZ"),
        schedule.dtend.format("%Y%m%dT%H%M%SZ"),
        schedule.summary,
        chrono::Utc::now().format("%Y%m%dT%H%M%SZ"),
        chrono::Utc::now().format("%Y%m%dT%H%M%SZ"),
    );

    if let Some(ref desc) = schedule.description {
        ics.push_str(&format!("DESCRIPTION:{}\n", desc));
    }
    if let Some(ref loc) = schedule.location {
        ics.push_str(&format!("LOCATION:{}\n", loc));
    }
    if !schedule.categories.is_empty() {
        ics.push_str(&format!(
            "CATEGORIES:{}\n",
            schedule.categories.join(",")
        ));
    }
    if let Some(p) = schedule.priority {
        ics.push_str(&format!("PRIORITY:{}\n", p));
    }
    if let Some(rrule_str) = schedule.rrule.to_rrule() {
        ics.push_str(&format!("RRULE:{}\n", rrule_str));
    }
    if let Some(minutes) = schedule.reminder_minutes {
        ics.push_str(&format!(
            "BEGIN:VALARM\nACTION:DISPLAY\nDESCRIPTION:{}\nTRIGGER:-PT{}M\nEND:VALARM\n",
            schedule.summary, minutes
        ));
    }

    ics.push_str("END:VEVENT\nEND:VCALENDAR\n");
    ics
}
