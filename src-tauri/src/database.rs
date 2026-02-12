use crate::log_parser::{Bookmark, LogEntry, TestSession};
use rusqlite::{params, Connection, OptionalExtension, Result as SqlResult};

// Re-export SearchResult from the parent module
pub use crate::SearchResult;

pub struct DatabaseManager {
    conn: Connection,
}

impl DatabaseManager {
    pub fn new(db_path: &str) -> SqlResult<Self> {
        let conn = Connection::open(db_path)?;
        Self::init_tables(&conn)?;
        Ok(Self { conn })
    }

    fn init_tables(conn: &Connection) -> SqlResult<()> {
        // Create test sessions table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS test_sessions (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                directory_path TEXT NOT NULL,
                file_count INTEGER DEFAULT 0,
                total_entries INTEGER DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                last_parsed_at DATETIME
            )",
            [],
        )?;

        // Create log entries table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS log_entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                test_session_id TEXT NOT NULL,
                file_path TEXT NOT NULL,
                file_index INTEGER NOT NULL,
                timestamp TEXT NOT NULL,
                level TEXT NOT NULL,
                stack TEXT NOT NULL,
                message TEXT NOT NULL,
                line_number INTEGER NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (test_session_id) REFERENCES test_sessions(id)
            )",
            [],
        )?;

        // Create bookmarks table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS bookmarks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                log_entry_id INTEGER NOT NULL,
                title TEXT,
                notes TEXT,
                color TEXT DEFAULT 'yellow',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (log_entry_id) REFERENCES log_entries(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Create indexes for performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_entries_session ON log_entries(test_session_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_entries_timestamp ON log_entries(timestamp)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_entries_level ON log_entries(level)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_bookmarks_entry ON bookmarks(log_entry_id)",
            [],
        )?;

        // Add source_type column if not exists (for existing databases)
        conn.execute(
            "ALTER TABLE test_sessions ADD COLUMN source_type TEXT DEFAULT 'local'",
            [],
        ).ok();

        Ok(())
    }

    pub fn create_test_session(&self, session: &TestSession) -> SqlResult<String> {
        self.conn.execute(
            "INSERT INTO test_sessions (id, name, directory_path, file_count, total_entries, source_type, last_parsed_at)
             VALUES (?, ?, ?, ?, ?, ?, datetime('now'))",
            params![
                &session.id,
                &session.name,
                &session.directory_path,
                session.file_count,
                session.total_entries,
                session.source_type.as_deref().unwrap_or("local")
            ],
        )?;
        Ok(session.id.clone())
    }

    pub fn insert_entries(&mut self, entries: &[LogEntry]) -> SqlResult<Vec<i64>> {
        let tx = self.conn.transaction()?;

        let mut inserted_ids = Vec::new();

        {
            let mut stmt = tx.prepare(
                "INSERT INTO log_entries
                 (test_session_id, file_path, file_index, timestamp, level, stack, message, line_number)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
            )?;

            for entry in entries {
                stmt.execute(params![
                    &entry.test_session_id,
                    &entry.file_path,
                    &entry.file_index,
                    &entry.timestamp,
                    &entry.level,
                    &entry.stack,
                    &entry.message,
                    &entry.line_number
                ])?;

                // Get the last inserted row ID from the transaction
                inserted_ids.push(tx.last_insert_rowid());
            }
        }

        tx.commit()?;
        Ok(inserted_ids)
    }

    pub fn get_entries_paginated(
        &self,
        session_id: &str,
        offset: usize,
        limit: usize,
        level_filter: Option<&[String]>, // Changed to &[String] for multi-select
        search_term: Option<&str>,
    ) -> SqlResult<(Vec<LogEntry>, usize)> {
        // Build query dynamically
        let mut base_query =
            "SELECT id, file_path, file_index, timestamp, level, stack, message, line_number
                             FROM log_entries WHERE test_session_id = ?"
                .to_string();

        let mut where_conditions: Vec<String> = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(session_id)];

        // Handle level filter - support multiple levels
        if let Some(levels) = level_filter {
            // Empty array means no levels selected - return no results
            if levels.is_empty() {
                // Add a condition that never matches
                where_conditions.push("1 = 0".to_string());
            } else {
                // Filter out empty strings and "ALL" (legacy support)
                let filtered_levels: Vec<&String> = levels.iter()
                    .filter(|level| !level.is_empty() && *level != "ALL")
                    .collect();

                if !filtered_levels.is_empty() {
                    // Build OR conditions for multiple levels using IN clause
                    let level_placeholders: Vec<String> = (0..filtered_levels.len() * 2)
                        .map(|_| "?".to_string())
                        .collect();

                    where_conditions.push(format!("(level IN ({}) OR level IN ({}))",
                        level_placeholders[..filtered_levels.len()].join(", "),
                        level_placeholders[filtered_levels.len()..].join(", ")));

                    // Add parameters for each level (with and without brackets)
                    for level in &filtered_levels {
                        params.push(Box::new(level.to_string()));
                    }
                    for level in &filtered_levels {
                        params.push(Box::new(format!("[{}]", level)));
                    }
                }
            }
        }

        if let Some(search) = search_term {
            where_conditions.push("(timestamp LIKE ? OR message LIKE ?)".to_string());
            let search_pattern = format!("%{}%", search);
            params.push(Box::new(search_pattern.clone()));
            params.push(Box::new(search_pattern));
        }

        if !where_conditions.is_empty() {
            base_query.push_str(" AND ");
            base_query.push_str(&where_conditions.join(" AND "));
        }

        // Get total count
        let count_query = base_query.replace(
            "SELECT id, file_path, file_index, timestamp, level, stack, message, line_number",
            "SELECT COUNT(*)",
        );

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let total: usize = self
            .conn
            .query_row(&count_query, &param_refs[..], |row| row.get(0))?;

        // Add pagination with secondary sort by id for consistent ordering
        let mut query = base_query;
        query.push_str(" ORDER BY timestamp ASC, id ASC LIMIT ? OFFSET ?");
        params.push(Box::new(limit));
        params.push(Box::new(offset));

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let mut stmt = self.conn.prepare(&query)?;
        let entry_iter = stmt.query_map(&param_refs[..], |row| {
            Ok(LogEntry {
                id: Some(row.get(0)?),
                test_session_id: session_id.to_string(),
                file_path: row.get(1)?,
                file_index: row.get(2)?,
                timestamp: row.get(3)?,
                level: row.get(4)?,
                stack: row.get(5)?,
                message: row.get(6)?,
                line_number: row.get(7)?,
                created_at: None,
            })
        })?;

        let entries: Result<Vec<_>, _> = entry_iter.collect();
        Ok((entries?, total))
    }

    pub fn add_bookmark(&self, bookmark: &Bookmark) -> SqlResult<i64> {
        self.conn.execute(
            "INSERT INTO bookmarks (log_entry_id, title, notes, color)
             VALUES (?, ?, ?, ?)",
            params![
                &bookmark.log_entry_id,
                &bookmark.title,
                &bookmark.notes,
                &bookmark.color,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_bookmarks(&self, session_id: &str) -> SqlResult<Vec<(Bookmark, LogEntry)>> {
        let query = "
            SELECT b.id, b.log_entry_id, b.title, b.notes, b.color, b.created_at,
                   e.id, e.test_session_id, e.file_path, e.file_index,
                   e.timestamp, e.level, e.stack, e.message, e.line_number
            FROM bookmarks b
            JOIN log_entries e ON b.log_entry_id = e.id
            WHERE e.test_session_id = ?
            ORDER BY e.timestamp ASC
        ";

        let mut stmt = self.conn.prepare(query)?;
        let bookmark_iter = stmt.query_map([session_id], |row| {
            let bookmark = Bookmark {
                id: Some(row.get(0)?),     // b.id
                log_entry_id: row.get(1)?,  // b.log_entry_id
                title: row.get(2)?,         // b.title
                notes: row.get(3)?,         // b.notes
                color: row.get(4)?,         // b.color
                created_at: None,           // b.created_at - skip for now
            };

            let entry = LogEntry {
                id: row.get(6).ok(),              // e.id
                test_session_id: row.get(7)?,     // e.test_session_id
                file_path: row.get(8)?,           // e.file_path
                file_index: row.get(9)?,          // e.file_index
                timestamp: row.get(10)?,          // e.timestamp (CORRECTED!)
                level: row.get(11)?,              // e.level (CORRECTED!)
                stack: row.get(12)?,              // e.stack
                message: row.get(13)?,            // e.message
                line_number: row.get(14)?,        // e.line_number
                created_at: None,
            };

            Ok((bookmark, entry))
        })?;

        bookmark_iter.collect()
    }

    pub fn get_sessions(&self) -> SqlResult<Vec<TestSession>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, directory_path, file_count, total_entries, created_at, last_parsed_at, source_type
             FROM test_sessions ORDER BY last_parsed_at DESC",
        )?;

        let session_iter = stmt.query_map([], |row| {
            Ok(TestSession {
                id: row.get(0)?,
                name: row.get(1)?,
                directory_path: row.get(2)?,
                file_count: row.get(3)?,
                total_entries: row.get(4)?,
                created_at: None,
                last_parsed_at: None,
                source_type: row.get(7)?,
            })
        })?;

        session_iter.collect()
    }

    pub fn delete_session(&self, session_id: &str) -> SqlResult<()> {
        log::info!("[DB] Starting delete_session for id={}", session_id);

        // Use transaction for atomicity
        let tx = self.conn.unchecked_transaction()?;

        {
            // First, get all log_entry_ids for this session
            let mut stmt = tx.prepare("SELECT id FROM log_entries WHERE test_session_id = ?")?;
            let entry_ids: Vec<i64> = stmt.query_map([session_id], |row| row.get(0))?
                .collect::<Result<Vec<_>, _>>()?;

            log::info!("[DB] Found {} log entries to delete", entry_ids.len());

            // Delete bookmarks for each entry
            if !entry_ids.is_empty() {
                for entry_id in &entry_ids {
                    tx.execute("DELETE FROM bookmarks WHERE log_entry_id = ?", [entry_id])?;
                }
                log::info!("[DB] Deleted bookmarks for {} entries", entry_ids.len());
            }

            // Delete log entries
            tx.execute(
                "DELETE FROM log_entries WHERE test_session_id = ?",
                [session_id],
            )?;
            log::info!("[DB] Deleted log entries for session");

            // Delete the session
            tx.execute("DELETE FROM test_sessions WHERE id = ?", [session_id])?;
            log::info!("[DB] Deleted session record");
        } // stmt is dropped here

        tx.commit()?;
        log::info!("[DB] Transaction committed");

        Ok(())
    }

    /// Find and delete a session with the same name and directory path
    /// Returns the session_id of the deleted session, if any
    pub fn delete_session_by_name_and_path(&self, name: &str, directory_path: &str) -> SqlResult<Option<String>> {
        log::info!("[DB] delete_session_by_name_and_path: name={}, dir={}", name, directory_path);

        let mut stmt = self.conn.prepare(
            "SELECT id FROM test_sessions WHERE name = ? AND directory_path = ?"
        )?;

        let session_id_opt = stmt.query_row(params![name, directory_path], |row| {
            row.get::<_, String>(0)
        }).optional()?;

        match &session_id_opt {
            Some(id) => println!("[DB] Found existing session: {}", id),
            None => log::info!("[DB] No existing session found"),
        }

        if let Some(session_id) = session_id_opt {
            self.delete_session(&session_id)?;
            Ok(Some(session_id))
        } else {
            Ok(None)
        }
    }

    pub fn delete_bookmark(&self, bookmark_id: i64) -> SqlResult<()> {
        self.conn
            .execute("DELETE FROM bookmarks WHERE id = ?", [bookmark_id])?;
        Ok(())
    }

    pub fn update_bookmark_title(&self, bookmark_id: i64, title: &str) -> SqlResult<()> {
        self.conn.execute(
            "UPDATE bookmarks SET title = ? WHERE id = ?",
            params![title, bookmark_id],
        )?;
        Ok(())
    }

    pub fn get_entry_page(
        &self,
        entry_id: i64,
        items_per_page: usize,
        level_filter: Option<&[String]>, // Changed to &[String] for multi-select
        search_term: Option<&str>,
    ) -> SqlResult<Option<usize>> {
        // First get the session_id, timestamp, and id for this entry
        let entry_info: Option<(String, String, i64)> = self
            .conn
            .query_row(
                "SELECT test_session_id, timestamp, id FROM log_entries WHERE id = ?",
                [entry_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?;

        let (session_id, entry_timestamp, entry_id_value) = match entry_info {
            Some(info) => info,
            None => return Ok(None), // Entry not found
        };

        // Build the same WHERE conditions as get_entries_paginated
        let mut where_conditions: Vec<String> = vec!["test_session_id = ?".to_string()];
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(session_id)];

        // Handle level filter - support multiple levels
        if let Some(levels) = level_filter {
            // Empty array means no levels selected - return no results
            if levels.is_empty() {
                // Add a condition that never matches
                where_conditions.push("1 = 0".to_string());
            } else {
                // Filter out empty strings and "ALL" (legacy support)
                let filtered_levels: Vec<&String> = levels.iter()
                    .filter(|level| !level.is_empty() && *level != "ALL")
                    .collect();

                if !filtered_levels.is_empty() {
                    // Build OR conditions for multiple levels using IN clause
                    let level_placeholders: Vec<String> = (0..filtered_levels.len() * 2)
                        .map(|_| "?".to_string())
                        .collect();

                    where_conditions.push(format!("(level IN ({}) OR level IN ({}))",
                        level_placeholders[..filtered_levels.len()].join(", "),
                        level_placeholders[filtered_levels.len()..].join(", ")));

                    // Add parameters for each level (with and without brackets)
                    for level in &filtered_levels {
                        params.push(Box::new(level.to_string()));
                    }
                    for level in &filtered_levels {
                        params.push(Box::new(format!("[{}]", level)));
                    }
                }
            }
        }

        if let Some(search) = search_term {
            where_conditions.push("(timestamp LIKE ? OR message LIKE ?)".to_string());
            let search_pattern = format!("%{}%", search);
            params.push(Box::new(search_pattern.clone()));
            params.push(Box::new(search_pattern));
        }

        // Count entries that come before this entry (same ordering: timestamp ASC, id ASC)
        // This matches entries with (timestamp < entry_timestamp) OR (timestamp = entry_timestamp AND id < entry_id)
        where_conditions.push("((timestamp < ?) OR (timestamp = ? AND id < ?))".to_string());
        params.push(Box::new(entry_timestamp.clone()));
        params.push(Box::new(entry_timestamp));
        params.push(Box::new(entry_id_value));

        let count_query = format!(
            "SELECT COUNT(*) FROM log_entries WHERE {}",
            where_conditions.join(" AND ")
        );

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let count_before: usize = self
            .conn
            .query_row(&count_query, &param_refs[..], |row| row.get(0))?;

        // Calculate page number (1-indexed)
        let page = (count_before / items_per_page) + 1;
        Ok(Some(page))
    }

    pub fn get_session_log_levels(&self, session_id: &str) -> SqlResult<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT level FROM log_entries WHERE test_session_id = ? ORDER BY level"
        )?;

        let level_iter = stmt.query_map([session_id], |row| row.get(0))?;
        level_iter.collect()
    }

    /// Get the page number for a specific log entry without filters (for search result jumping).
    ///
    /// Counts entries that come before the target entry in the same session,
    /// ordered by timestamp ASC, id ASC.
    pub fn find_entry_page_simple(
        &self,
        session_id: &str,
        entry_id: i64,
        items_per_page: usize,
    ) -> SqlResult<usize> {
        // First get the timestamp for the target entry
        let entry_timestamp: String = self.conn.query_row(
            "SELECT timestamp FROM log_entries WHERE test_session_id = ? AND id = ?",
            [session_id, &entry_id.to_string()],
            |row| row.get(0),
        )?;

        // Count entries before this one (same ordering as get_log_entries: timestamp ASC, id ASC)
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM log_entries
             WHERE test_session_id = ? AND ((timestamp < ?) OR (timestamp = ? AND id < ?))",
            [session_id, &entry_timestamp, &entry_timestamp, &entry_id.to_string()],
            |row| row.get(0),
        )?;

        let page = (count as usize) / items_per_page + 1;
        Ok(page)
    }

    /// Query entries with failure anchor markers for a session.
    ///
    /// Returns entry IDs where message ends with [FAIL] marker.
    fn query_failure_entries(&self, session_id: &str) -> SqlResult<Vec<i64>> {
        let mut stmt = self.conn.prepare(
            "SELECT id FROM log_entries
             WHERE test_session_id = ? AND message LIKE '%[FAIL]'
             ORDER BY timestamp ASC"
        )?;

        let entry_iter = stmt.query_map([session_id], |row| {
            Ok(row.get::<_, i64>(0)?)
        })?;

        entry_iter.collect()
    }

    /// Get existing bookmark for a specific entry.
    ///
    /// Returns the bookmark and its associated log entry if found.
    fn get_bookmark_for_entry(&self, entry_id: i64) -> SqlResult<Option<(Bookmark, LogEntry)>> {
        let query = "
            SELECT b.id, b.log_entry_id, b.title, b.notes, b.color,
                   e.id, e.test_session_id, e.file_path, e.file_index,
                   e.timestamp, e.level, e.stack, e.message, e.line_number
            FROM bookmarks b
            JOIN log_entries e ON b.log_entry_id = e.id
            WHERE b.log_entry_id = ?
        ";

        let mut stmt = self.conn.prepare(query)?;

        let result = stmt.query_row([entry_id], |row| {
            let bookmark = Bookmark {
                id: Some(row.get(0)?),
                log_entry_id: row.get(1)?,
                title: row.get(2)?,
                notes: row.get(3)?,
                color: row.get(4)?,
                created_at: None,
            };

            let entry = LogEntry {
                id: row.get(5).ok(),
                test_session_id: row.get(6)?,
                file_path: row.get(7)?,
                file_index: row.get(8)?,
                timestamp: row.get(9)?,
                level: row.get(10)?,
                stack: row.get(11)?,
                message: row.get(12)?,
                line_number: row.get(13)?,
                created_at: None,
            };

            Ok((bookmark, entry))
        });

        match result {
            Ok(pair) => Ok(Some(pair)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Update bookmark color and title.
    ///
    /// Used to upgrade existing bookmarks to failure status.
    fn update_bookmark_color_and_title(&self, bookmark_id: i64, color: &str, title: &str) -> SqlResult<()> {
        self.conn.execute(
            "UPDATE bookmarks SET color = ?, title = ? WHERE id = ?",
            params![color, title, bookmark_id],
        )?;
        Ok(())
    }

    /// Auto-bookmark entries for a session that haven't been bookmarked yet.
    /// This is called when switching to a session to ensure bookmarks are created.
    ///
    /// Priority order:
    /// 1. Failure anchors ([FAIL] marker) - Red (#F56C6C), "Failure" title
    /// 2. STEP entries (MARKER level with [STEP in message) - Dark Turquoise (#00CED1)
    /// 3. ###TEXT### pattern - Transparent background
    /// 4. MARKER level (other) - Transparent background
    ///
    /// Higher priority bookmarks will override lower priority ones if already bookmarked.
    pub fn ensure_auto_bookmarks_for_session(&self, session_id: &str) -> SqlResult<Vec<Bookmark>> {
        use regex::Regex;

        let re = Regex::new(r"###(.+?)###").unwrap();
        let mut created_bookmarks = Vec::new();

        // === PRIORITY 1: Failure Anchors ===
        let failure_entry_ids = self.query_failure_entries(session_id)?;
        for entry_id in failure_entry_ids {
            if let Some((existing_bookmark, _)) = self.get_bookmark_for_entry(entry_id)? {
                if existing_bookmark.color != Some("#F56C6C".to_string()) {
                    self.update_bookmark_color_and_title(
                        existing_bookmark.id.unwrap(),
                        "#F56C6C",
                        "Failure",
                    )?;
                    let mut updated = existing_bookmark.clone();
                    updated.color = Some("#F56C6C".to_string());
                    updated.title = Some("Failure".to_string());
                    created_bookmarks.push(updated);
                }
            } else {
                let bookmark = Bookmark {
                    id: None,
                    log_entry_id: entry_id,
                    title: Some("Failure".to_string()),
                    notes: None,
                    color: Some("#F56C6C".to_string()),
                    created_at: None,
                };
                let bookmark_id = self.add_bookmark(&bookmark)?;
                let mut created = bookmark;
                created.id = Some(bookmark_id);
                created_bookmarks.push(created);
            }
        }

        // === PRIORITY 2: STEP Entries ===
        // Query MARKER level entries with [STEP in message (but not failures)
        let step_query = "
            SELECT e.id, e.message,
                   (SELECT COUNT(*) FROM bookmarks b WHERE b.log_entry_id = e.id) as has_bookmark
            FROM log_entries e
            WHERE e.test_session_id = ?
              AND e.level = 'MARKER'
              AND e.message LIKE '%[STEP%'
              AND e.message NOT LIKE '%[FAIL]'
            ORDER BY e.timestamp ASC
        ";

        {
            let mut stmt = self.conn.prepare(step_query)?;
            let step_iter = stmt.query_map([session_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })?;

            for step_result in step_iter {
                let (entry_id, message, has_bookmark) = step_result?;

                if has_bookmark > 0 {
                    // Check if it has the right color, update if not
                    if let Some((existing_bookmark, _)) = self.get_bookmark_for_entry(entry_id)? {
                        if existing_bookmark.color != Some("#00CED1".to_string()) {
                            self.update_bookmark_color_and_title(
                                existing_bookmark.id.unwrap(),
                                "#00CED1",
                                &message,
                            )?;
                            let mut updated = existing_bookmark.clone();
                            updated.color = Some("#00CED1".to_string());
                            updated.title = Some(message.clone());
                            created_bookmarks.push(updated);
                        }
                    }
                } else {
                    // Create new STEP bookmark
                    let bookmark = Bookmark {
                        id: None,
                        log_entry_id: entry_id,
                        title: Some(message.clone()),
                        notes: None,
                        color: Some("#00CED1".to_string()), // Dark Turquoise
                        created_at: None,
                    };
                    let bookmark_id = self.add_bookmark(&bookmark)?;
                    let mut created = bookmark;
                    created.id = Some(bookmark_id);
                    created_bookmarks.push(created);
                }
            }
        }

        // === PRIORITY 3 & 4: ###TEXT### pattern and other MARKER level ===
        // Skip entries that are already bookmarked (including STEP and Failure)
        let query = "
            SELECT e.id, e.message, e.level,
                   (SELECT COUNT(*) FROM bookmarks b WHERE b.log_entry_id = e.id) as has_bookmark
            FROM log_entries e
            WHERE e.test_session_id = ?
              AND (e.message LIKE '%###%' OR e.level = 'MARKER')
              AND e.message NOT LIKE '%[FAIL]'
              AND e.message NOT LIKE '%[STEP%'
            ORDER BY e.timestamp ASC
        ";

        let mut stmt = self.conn.prepare(query)?;
        let entry_iter = stmt.query_map([session_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })?;

        for entry_result in entry_iter {
            let (entry_id, message, level, has_bookmark) = entry_result?;

            if has_bookmark > 0 {
                continue;
            }

            let title = if let Some(caps) = re.captures(&message) {
                if let Some(title_match) = caps.get(1) {
                    let extracted = title_match.as_str().trim().to_string();
                    if !extracted.is_empty() {
                        Some(extracted)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            let final_title = if let Some(t) = title {
                t
            } else if level == "MARKER" {
                message.clone()
            } else {
                continue;
            };

            let bookmark = Bookmark {
                id: None,
                log_entry_id: entry_id,
                title: Some(final_title),
                notes: None,
                color: None,
                created_at: None,
            };

            let bookmark_id = self.add_bookmark(&bookmark)?;
            let mut created = bookmark;
            created.id = Some(bookmark_id);
            created_bookmarks.push(created);
        }

        Ok(created_bookmarks)
    }

    pub fn search_entries_custom(&self, query: &str, params: &[Box<dyn rusqlite::ToSql>]) -> SqlResult<Vec<SearchResult>> {
        let mut stmt = self.conn.prepare(query)?;

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let mut rows = stmt.query(&param_refs[..])?;

        let mut results = Vec::new();
        while let Some(row) = rows.next()? {
            results.push(SearchResult {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                line_number: row.get(2)?,
                message: row.get(3)?,
            });
        }

        Ok(results)
    }
}
