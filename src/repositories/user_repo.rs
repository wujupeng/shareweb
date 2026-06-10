use rusqlite::{params, Connection, Result as SqlResult};
use crate::models::user::{User, UserRole, UserStatus};

pub struct UserRepo;

impl UserRepo {
    pub fn find_by_username(conn: &Connection, username: &str) -> SqlResult<Option<User>> {
        let mut stmt = conn.prepare(
            "SELECT id, username, password_hash, role, status, login_fail_count, locked_until, created_at, updated_at FROM users WHERE username = ?1"
        )?;
        let result = stmt.query_row(params![username], |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                password_hash: row.get(2)?,
                role: UserRole::from_str(&row.get::<_, String>(3)?).unwrap_or(UserRole::ReadOnly),
                status: UserStatus::from_str(&row.get::<_, String>(4)?).unwrap_or(UserStatus::Active),
                login_fail_count: row.get(5)?,
                locked_until: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        });
        match result {
            Ok(user) => Ok(Some(user)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn create(conn: &Connection, username: &str, password_hash: &str, role: &str) -> SqlResult<()> {
        conn.execute(
            "INSERT INTO users (username, password_hash, role) VALUES (?1, ?2, ?3)",
            params![username, password_hash, role],
        )?;
        Ok(())
    }

    pub fn update_login_fail(conn: &Connection, username: &str, fail_count: i32, locked_until: Option<&str>) -> SqlResult<()> {
        conn.execute(
            "UPDATE users SET login_fail_count = ?1, locked_until = ?2, updated_at = datetime('now') WHERE username = ?3",
            params![fail_count, locked_until, username],
        )?;
        Ok(())
    }

    pub fn update_password(conn: &Connection, username: &str, password_hash: &str) -> SqlResult<()> {
        conn.execute(
            "UPDATE users SET password_hash = ?1, updated_at = datetime('now') WHERE username = ?2",
            params![password_hash, username],
        )?;
        Ok(())
    }

    pub fn update_role_and_status(conn: &Connection, username: &str, role: &str, status: &str) -> SqlResult<()> {
        conn.execute(
            "UPDATE users SET role = ?1, status = ?2, updated_at = datetime('now') WHERE username = ?3",
            params![role, status, username],
        )?;
        Ok(())
    }

    pub fn delete(conn: &Connection, username: &str) -> SqlResult<()> {
        conn.execute("DELETE FROM users WHERE username = ?1", params![username])?;
        Ok(())
    }

    pub fn list(conn: &Connection, status_filter: Option<&str>, limit: i64, offset: i64) -> SqlResult<Vec<User>> {
        let (sql, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = match status_filter {
            Some(s) => (
                "SELECT id, username, password_hash, role, status, login_fail_count, locked_until, created_at, updated_at FROM users WHERE status = ?1 LIMIT ?2 OFFSET ?3",
                vec![Box::new(s.to_string()), Box::new(limit), Box::new(offset)],
            ),
            None => (
                "SELECT id, username, password_hash, role, status, login_fail_count, locked_until, created_at, updated_at FROM users LIMIT ?1 OFFSET ?2",
                vec![Box::new(limit), Box::new(offset)],
            ),
        };
        let mut stmt = conn.prepare(sql)?;
        let p: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let mut rows = stmt.query(p.as_slice())?;
        let mut users = Vec::new();
        let mut rows = rows;
        while let Some(row) = rows.next()? {
            users.push(User {
                id: row.get(0)?,
                username: row.get(1)?,
                password_hash: row.get(2)?,
                role: UserRole::from_str(&row.get::<_, String>(3)?).unwrap_or(UserRole::ReadOnly),
                status: UserStatus::from_str(&row.get::<_, String>(4)?).unwrap_or(UserStatus::Active),
                login_fail_count: row.get(5)?,
                locked_until: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            });
        }
        Ok(users)
    }

    pub fn count_admins(conn: &Connection) -> SqlResult<i64> {
        conn.query_row("SELECT COUNT(*) FROM users WHERE role = 'admin'", [], |row| row.get(0))
    }

    pub fn init_admin(conn: &Connection, password_hash: &str) -> SqlResult<()> {
        conn.execute(
            "INSERT INTO users (username, password_hash, role) VALUES ('admin', ?1, 'admin')
             ON CONFLICT(username) DO UPDATE SET password_hash = ?1",
            params![password_hash],
        )?;
        Ok(())
    }
}
