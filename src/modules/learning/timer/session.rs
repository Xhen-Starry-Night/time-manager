use crate::data::models::{SessionInsert, SessionParams};
use crate::modules::learning::timer::state_machine::StoppedSession;

pub fn create_session_insert(stopped: StoppedSession, params: SessionParams, note: Option<String>) -> SessionInsert {
    SessionInsert {
        category_id: stopped.category_id,
        start_time: stopped.start_time,
        end_time: stopped.end_time,
        duration_secs: stopped.duration_secs,
        pause_records: stopped.pause_records,
        params,
        note,
    }
}