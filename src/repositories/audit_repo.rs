use rusqlite::{params, Connection, Result as SqlResult};
use crate::models::audit::{AuditLog, AuditActionType, AuditResult};

pub struct AuditRepo;

impl AuditRepo {
    pub fn insert(
        conn: &Connection,
        operator: &str,
        action_type: &str,
        target_path: Option<&str>,
        detail: Option<&str>,
        source_ip: &str,
        result: &str,
        failure_reason: Option<&str>,
    ) -> SqlResult<()> {
        conn.execute(
            "INSERT INTO audit_logs (operator, action_type, target_path, detail, source_ip, result, failure_reason) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![operator, action_type, target_path, detail, source_ip, result, failure_reason],
        )?;
        Ok(())
    }

    pub fn list(
        conn: &Connection,
        operator_filter: Option<&str>,
        action_type_filter: Option<&str>,
        start_time: Option<&str>,
        end_time: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> SqlResult<Vec<AuditLog>> {
        let mut sql = String::from(
            "SELECT id, operator, action_type, target_path, detail, source_ip, result, failure_reason, action_time FROM audit_logs WHERE 1=1"
        );
        let mut param_idx = 1;
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(op) = operator_filter {
            sql.push_str(&format!(" AND operator = ?{}", param_idx));
            param_values.push(Box::new(op.to_string()));
            param_idx += 1;
        }
        if let Some(at) = action_type_filter {
            sql.push_str(&format!(" AND action_type = ?{}", param_idx));
            param_values.push(Box::new(at.to_string()));
            param_idx += 1;
        }
        if let Some(st) = start_time {
            sql.push_str(&format!(" AND action_time >= ?{}", param_idx));
            param_values.push(Box::new(st.to_string()));
            param_idx += 1;
        }
        if let Some(et) = end_time {
            sql.push_str(&format!(" AND action_time <= ?{}", param_idx));
            param_values.push(Box::new(et.to_string()));
            param_idx += 1;
        }
        sql.push_str(&format!(" ORDER BY action_time DESC LIMIT ?{} OFFSET ?{}", param_idx, param_idx + 1));

        let params: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).chain(
            vec![&limit as &dyn rusqlite::types::ToSql, &offset as &dyn rusqlite::types::ToSql].into_iter()
        ).collect();

        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query(params.as_slice())?;
        let mut logs = Vec::new();
        while let Some(row) = rows.next()? {
            logs.push(AuditLog {
                id: row.get(0)?,
                operator: row.get(1)?,
                action_type: AuditActionType::from_str(&row.get::<_, String>(2)?).unwrap_or(AuditActionType::Login),
                target_path: row.get(3)?,
                detail: row.get(4)?,
                source_ip: row.get(5)?,
                result: AuditResult::from_str(&row.get::<_, String>(6)?).unwrap_or(AuditResult::Success),
                failure_reason: row.get(7)?,
                action_time: row.get(8)?,
            });
        }
        Ok(logs)
    }
}
