-- migrations/cleopatra.sql

-- =========================================================
-- execution: a run / suite which groups test results
-- =========================================================
CREATE TABLE IF NOT EXISTS execution (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    tag TEXT,
    created_by TEXT,
    time_created INTEGER NOT NULL
);

-- =========================================================
-- test_result: each row is one test case result within an execution
-- =========================================================
CREATE TABLE IF NOT EXISTS test_result (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    execution_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    platform TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL,
    execution_time INTEGER,
    counter INTEGER NOT NULL DEFAULT 1,
    log TEXT,
    screenshot_id INTEGER,
    created_by TEXT,
    time_created INTEGER NOT NULL,
    CONSTRAINT uq_test_result_execution_name UNIQUE (execution_id, name)
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_test_result_execution_id ON test_result (execution_id);
CREATE INDEX IF NOT EXISTS idx_execution_name ON execution (name);
