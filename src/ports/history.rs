use crate::models::ProjectRecord;
use anyhow::Result;

pub(crate) trait ProjectHistoryStore {
    fn load(&self) -> Result<Option<ProjectRecord>>;
    fn save(&self, record: &ProjectRecord) -> Result<()>;
}
