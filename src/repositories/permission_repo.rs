use rusqlite::{params, Connection, Result as SqlResult};
use crate::models::permission::{PermissionRule, ActionType};
use std::str::FromStr;

pub struct PermissionRepo;

impl PermissionRepo {
    pub fn create(conn: &Connection, path: &str, role: &str, allowed_actions: &str, inherit: bool) -> SqlResult<()> {
        conn.execute(
            "INSERT INTO permission_rules (path, role, allowed_actions, inherit) VALUES (?1, ?2, ?3, ?4)",
            params![path, role, allowed_actions, inherit as i32],
        )?;
        Ok(())
    }

    pub fn list(conn: &Connection, path_filter: Option<&str>) -> SqlResult<Vec<PermissionRule>> {
        let sql = match path_filter {
            Some(_) => "SELECT id, path, role, allowed_actions, inherit, created_at, updated_at FROM permission_rules WHERE path LIKE ?1",
            None => "SELECT id, path, role, allowed_actions, inherit, created_at, updated_at FROM permission_rules",
        };
        let mut stmt = conn.prepare(sql)?;
        let mut rows = match path_filter {
            Some(p) => stmt.query(params![format!("{}%", p)])?,
            None => stmt.query([])?,
        };
        let mut rules = Vec::new();
        while let Some(row) = rows.next()? {
            let actions_str: String = row.get(3)?;
            let actions: Vec<ActionType> = serde_json::from_str(&actions_str)
                .unwrap_or_default();
            rules.push(PermissionRule {
                id: row.get(0)?,
                path: row.get(1)?,
                role: row.get(2)?,
                allowed_actions: actions,
                inherit: row.get::<_, i32>(4)? != 0,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            });
        }
        Ok(rules)
    }

    pub fn find_by_path(conn: &Connection, path: &str) -> SqlResult<Vec<PermissionRule>> {
        let mut stmt = conn.prepare(
            "SELECT id, path, role, allowed_actions, inherit, created_at, updated_at FROM permission_rules WHERE path = ?1"
        )?;
        let mut rows = stmt.query(params![path])?;
        let mut rules = Vec::new();
        while let Some(row) = rows.next()? {
            let actions_str: String = row.get(3)?;
            let actions: Vec<ActionType> = serde_json::from_str(&actions_str).unwrap_or_default();
            rules.push(PermissionRule {
                id: row.get(0)?,
                path: row.get(1)?,
                role: row.get(2)?,
                allowed_actions: actions,
                inherit: row.get::<_, i32>(4)? != 0,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            });
        }
        Ok(rules)
    }

    pub fn delete(conn: &Connection, id: i64) -> SqlResult<()> {
        conn.execute("DELETE FROM permission_rules WHERE id = ?1", params![id])?;
        Ok(())
    }
}
