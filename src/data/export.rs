use crate::data::error::Result;
use crate::data::models::{PredictionState, Session};
use std::io::Write;
use std::path::Path;

pub trait Exporter {
    fn export_sessions(&self, sessions: &[Session], output_dir: &Path) -> Result<std::path::PathBuf>;
    fn export_predictions(&self, predictions: &[PredictionState], output_dir: &Path) -> Result<std::path::PathBuf>;
}

pub struct JsonExporter;
pub struct CsvExporter;
pub struct MarkdownExporter;

impl Exporter for JsonExporter {
    fn export_sessions(&self, sessions: &[Session], output_dir: &Path) -> Result<std::path::PathBuf> {
        std::fs::create_dir_all(output_dir)?;
        let path = output_dir.join("sessions.jsonl");
        let mut file = std::fs::File::create(&path)?;
        for session in sessions {
            let line = serde_json::to_string(session)?;
            writeln!(file, "{}", line)?;
        }
        Ok(path)
    }

    fn export_predictions(&self, predictions: &[PredictionState], output_dir: &Path) -> Result<std::path::PathBuf> {
        std::fs::create_dir_all(output_dir)?;
        let path = output_dir.join("prediction_states.jsonl");
        let mut file = std::fs::File::create(&path)?;
        for pred in predictions {
            let line = serde_json::to_string(pred)?;
            writeln!(file, "{}", line)?;
        }
        Ok(path)
    }
}

impl Exporter for CsvExporter {
    fn export_sessions(&self, sessions: &[Session], output_dir: &Path) -> Result<std::path::PathBuf> {
        std::fs::create_dir_all(output_dir)?;
        let path = output_dir.join("sessions.csv");
        let mut file = std::fs::File::create(&path)?;
        writeln!(file, "id,category_id,start_time,end_time,duration_secs,quality,understanding_difficulty,memory_difficulty,completion_rate,note,is_manual_edit")?;
        for s in sessions {
            writeln!(
                file,
                "{},{},{},{},{},{},{},{},{},{},{}",
                s.id,
                s.category_id,
                s.start_time.to_rfc3339(),
                s.end_time.to_rfc3339(),
                s.duration_secs,
                s.quality,
                s.understanding_difficulty,
                s.memory_difficulty,
                s.completion_rate,
                s.note.as_deref().unwrap_or(""),
                s.is_manual_edit as i32,
            )?;
        }
        Ok(path)
    }

    fn export_predictions(&self, predictions: &[PredictionState], output_dir: &Path) -> Result<std::path::PathBuf> {
        std::fs::create_dir_all(output_dir)?;
        let path = output_dir.join("prediction_states.csv");
        let mut file = std::fs::File::create(&path)?;
        writeln!(file, "id,category_id,algorithm,last_review,next_review")?;
        for p in predictions {
            writeln!(
                file,
                "{},{},{},{},{}",
                p.id, p.category_id, p.algorithm, p.last_review.to_rfc3339(), p.next_review.to_rfc3339()
            )?;
        }
        Ok(path)
    }
}

impl Exporter for MarkdownExporter {
    fn export_sessions(&self, sessions: &[Session], output_dir: &Path) -> Result<std::path::PathBuf> {
        std::fs::create_dir_all(output_dir)?;
        let path = output_dir.join("sessions.md");
        let mut file = std::fs::File::create(&path)?;
        writeln!(file, "# Sessions")?;
        writeln!(file)?;
        writeln!(file, "| ID | Category | Start | End | Duration | Quality | Note |")?;
        writeln!(file, "|----|----------|-------|-----|----------|---------|------|")?;
        for s in sessions {
            let dur = format_duration(s.duration_secs);
            writeln!(
                file,
                "| {} | {} | {} | {} | {} | {} | {} |",
                s.id,
                s.category_id,
                s.start_time.format("%Y-%m-%d %H:%M"),
                s.end_time.format("%Y-%m-%d %H:%M"),
                dur,
                s.quality,
                s.note.as_deref().unwrap_or("-"),
            )?;
        }
        Ok(path)
    }

    fn export_predictions(&self, predictions: &[PredictionState], output_dir: &Path) -> Result<std::path::PathBuf> {
        std::fs::create_dir_all(output_dir)?;
        let path = output_dir.join("prediction_states.md");
        let mut file = std::fs::File::create(&path)?;
        writeln!(file, "# Prediction States")?;
        writeln!(file)?;
        writeln!(file, "| Category | Algorithm | Last Review | Next Review |")?;
        writeln!(file, "|----------|-----------|-------------|-------------|")?;
        for p in predictions {
            writeln!(
                file,
                "| {} | {} | {} | {} |",
                p.category_id,
                p.algorithm,
                p.last_review.format("%Y-%m-%d"),
                p.next_review.format("%Y-%m-%d"),
            )?;
        }
        Ok(path)
    }
}

fn format_duration(secs: i64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}