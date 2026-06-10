use rusqlite::{params, Connection, Result as SqlResult};
use crate::models::upload_task::{UploadTask, UploadStatus};

pub struct UploadTaskRepo;

impl UploadTaskRepo {
    pub fn create(conn: &Connection, task: &UploadTask) -> SqlResult<()> {
        conn.execute(
            "INSERT INTO upload_tasks (id, file_name, target_path, total_size, chunk_size, total_chunks, uploaded_chunks, status) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![task.id, task.file_name, task.target_path, task.total_size, task.chunk_size, task.total_chunks, serde_json::to_string(&task.uploaded_chunks).unwrap_or_default(), task.status.as_str()],
        )?;
        Ok(())
    }

    pub fn find_by_id(conn: &Connection, id: &str) -> SqlResult<Option<UploadTask>> {
        let mut stmt = conn.prepare(
            "SELECT id, file_name, target_path, total_size, chunk_size, total_chunks, uploaded_chunks, status, created_at, updated_at FROM upload_tasks WHERE id = ?1"
        )?;
        let mut rows = stmt.query(params![id])?;
        match rows.next()? {
            Some(row) => {
                let chunks_str: String = row.get(6)?;
                Ok(Some(UploadTask {
                    id: row.get(0)?,
                    file_name: row.get(1)?,
                    target_path: row.get(2)?,
                    total_size: row.get(3)?,
                    chunk_size: row.get(4)?,
                    total_chunks: row.get(5)?,
                    uploaded_chunks: serde_json::from_str(&chunks_str).unwrap_or_default(),
                    status: UploadStatus::from_str(row.get::<_, String>(7)?.as_str()).unwrap_or(UploadStatus::Pending),
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                }))
            }
            None => Ok(None),
        }
    }

    pub fn update_status(conn: &Connection, id: &str, status: &str, uploaded_chunks: &str) -> SqlResult<()> {
        conn.execute(
            "UPDATE upload_tasks SET status = ?1, uploaded_chunks = ?2, updated_at = datetime('now') WHERE id = ?3",
            params![status, uploaded_chunks, id],
        )?;
        Ok(())
    }

    pub fn cleanup_expired(conn: &Connection, before_time: &str) -> SqlResult<usize> {
        Ok(conn.execute(
            "DELETE FROM upload_tasks WHERE status IN ('pending', 'failed', 'cancelled') AND updated_at < ?1",
            params![before_time],
        )?)
    }
}
