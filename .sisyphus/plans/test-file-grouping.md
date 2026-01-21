# Plan: Test File Grouping and Pattern Filtering

## Issue Summary

Currently, when opening a folder, the application parses **ALL** HTML files in the directory, including unrelated files like `MainRollup.html`. It should instead:
1. Parse only HTML files that belong to specific tests
2. Group files by their test name
3. Ignore files that don't match the test naming pattern

## File Naming Pattern

Based on the test file `TestEnableTcpdump_ID_1---0.html`, the pattern is:
```
<TestName>_<ID_X>---<Y>.html
```

Where:
- `<TestName>`: Test identifier (e.g., "TestEnableTcpdump", "TestABC")
- `<ID_X>`: Test instance ID (e.g., "ID_1")
- `<Y>`: Sequential file number (e.g., 0, 1, 2)

**Files belonging to the same test share the same TestName and ID_X.**

Examples:
- `TestEnableTcpdump_ID_1---0.html` → Test: TestEnableTcpdump_ID_1
- `TestEnableTcpdump_ID_1---1.html` → Test: TestEnableTcpdump_ID_1
- `TestABC_ID_1---0.html` → Test: TestABC_ID_1
- `MainRollup.html` → IGNORE (doesn't match pattern)

## Implementation Plan

### Phase 1: Backend Pattern Matching Logic

#### 1.1 Add Test File Validation Function

**File:** `src-tauri/src/log_parser.rs`

Add functions to validate and extract test names from filenames:
- `is_test_log_file(filename: &str) -> Option<String>`: Validates pattern, returns test name
- `extract_test_name(file_path: &str) -> Option<String>`: Extracts test name from full path

Pattern validation requirements:
- Must end with `.html`
- Must contain `---<number>.html` at the end
- Must contain `_ID_` pattern before the `---`
- Must not have `_ID_` at the very start of the filename

#### 1.2 Update `scan_html_files()` to Group Files

**File:** `src-tauri/src/log_parser.rs`

**Changes:**
- Return type: `Vec<String>` → `HashMap<String, Vec<String>>`
- Filter files using `extract_test_name()` to find test files
- Group files by test name in HashMap
- Log ignored files (for debugging)
- Sort files within each group by numeric index

#### 1.3 Update `parse_directory_blocking()` for Multiple Sessions

**File:** `src-tauri/src/lib.rs`

**Changes:**
- Return type: `(String, usize, usize)` → `Vec<(String, String, usize, usize)>`
  - Tuple: `(session_id, test_name, file_count, total_entries)`
- Create separate session for each test group
- Use test name as session name (instead of directory name)
- Return vector of all created sessions

#### 1.4 Update Tauri Command

**File:** `src-tauri/src/lib.rs`

**`parse_log_directory()` changes:**
- Return type: `Result<Vec<String>, String>` (vector of session IDs)
- Extract session IDs from session results

#### 1.5 Update Frontend

**File:** `src/App.vue`

**`openDirectory()` changes:**
- Handle array of session IDs instead of single ID
- Select first session automatically after loading
- Update loading messages to reflect multiple sessions

### Phase 2: Testing

#### 2.1 Unit Tests

**File:** `src-tauri/tests/log_parser_tests.rs`

Add tests for:
- `test_is_test_log_file_valid()`: Valid pattern matching
- `test_is_test_log_file_invalid()`: Invalid patterns (MainRollup.html, missing _ID_, etc.)
- `test_extract_test_name()`: Full path handling

#### 2.2 Integration Test

Add `test_scan_and_group_files()`:
- Create temporary directory with multiple test files
- Include files like MainRollup.html that should be ignored
- Verify correct grouping
- Verify ignored files are not included

### Phase 3: Documentation

#### 3.1 Update README.md

Add section documenting:
- Test log file naming convention
- Examples of valid and invalid files
- How sessions are created per test

#### 3.2 Code Comments

Add comprehensive doc comments for new functions.

## Testing Checklist

- [ ] Unit tests for pattern matching (valid/invalid cases)
- [ ] Integration test for file grouping
- [ ] Manual testing with real log directory
- [ ] Test with MainRollup.html present
- [ ] Test with nested subdirectories
- [ ] Verify session selection works
- [ ] Verify delete session works independently

## Edge Cases

1. Empty directory → Error message
2. Only invalid files → Error "No test log files found"
3. Files in subdirectories → Should still be grouped correctly
4. Case sensitivity → Match exactly as in filenames
5. Very long test names → Handle gracefully

## Migration Notes

**No database schema changes required.** The existing `test_sessions` table works with new implementation.

**Breaking change:** `parse_log_directory` command now returns `Vec<String>` instead of `String`.

## Summary

This plan implements intelligent test file grouping:
1. Validates test log file patterns
2. Groups files by test name
3. Creates separate sessions per test
4. Ignores unrelated files like MainRollup.html
5. Maintains database compatibility
6. Adds comprehensive tests
