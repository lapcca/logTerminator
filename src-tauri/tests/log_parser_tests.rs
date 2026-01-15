use logterminator_lib::log_parser::{HtmlLogParser, LogEntry};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_file_index() {
        assert_eq!(HtmlLogParser::extract_file_index("TestEnableTcpdump_ID_1---0.html"), 0);
        assert_eq!(HtmlLogParser::extract_file_index("TestEnableTcpdump_ID_1---10.html"), 10);
        assert_eq!(HtmlLogParser::extract_file_index("test---5.html"), 5);
        assert_eq!(HtmlLogParser::extract_file_index("no_index.html"), 0);
    }

    #[test]
    fn test_parse_simple_html() {
        let html = r#"
        <!DOCTYPE html>
        <html>
        <body>
        <table>
        <tr class="HEADER">
            <th>Timestamp</th>
            <th>Level</th>
            <th>Stack</th>
            <th>Message</th>
        </tr>
        <tr class="INFO">
            <td class="date">2026/01/14 07:17:37,370 UTC</td>
            <td class="level">[INFO]</td>
            <td class="hierarchy">Thread: &lt;MainThread&gt;</td>
            <td class="message"><pre class="INFO">Test message</pre></td>
        </tr>
        </table>
        </body>
        </html>
        "#;

        // Write to temp file
        std::fs::write("test_temp.html", html).expect("Failed to write test file");

        let result = HtmlLogParser::parse_file("test_temp.html", "test_session", 0);

        // Clean up
        let _ = std::fs::remove_file("test_temp.html");

        assert!(result.is_ok());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].level, "[INFO]");
        assert_eq!(entries[0].timestamp, "2026/01/14 07:17:37,370 UTC");
        assert!(entries[0].message.contains("Test message"));
    }

    #[test]
    fn test_parse_real_html_file() {
        // Create a test HTML file with the actual structure
        let html_content = r#"<!DOCTYPE html>
<html>
<head>
    <title>Test Log File</title>
</head>
<body>
<div id="content">
<table id="log">
    <tr class="HEADER">
        <th class="header-timestamp">Timestamp</th>
        <th class="header-level">Level</th>
        <th class="header-stack">Stack</th>
        <th class="header-message">Message</th>
    </tr>

<tr class="INFO"><td class="date">2026/01/14 07:17:37,370 UTC</td><td class="level">[INFO]</td><td class="hierarchy">Thread: &lt;MainThread&gt;
pytest_test_engine.pytest_test_engine -- 871</td><td class="message"><pre class="INFO">Package version info:
atlas-framework: 7.0.378
atlas-unity: 1.0.52</pre></td></tr>

<tr class="DEBUG"><td class="date">2026/01/14 07:17:37,374 UTC</td><td class="level">[DEBUG]</td><td class="hierarchy">Thread: &lt;MainThread&gt;
pytest_test_engine.pytest_test_engine -- 1197</td><td class="message"><pre class="DEBUG">Create data collection folder</pre></td></tr>

<tr class="WARNING"><td class="date">2026/01/14 07:17:37,375 UTC</td><td class="level">[WARNING]</td><td class="hierarchy">Thread: &lt;MainThread&gt;
atlas_framework.ctd_testbed.base_services -- 347</td><td class="message"><pre class="WARNING">The PYTEST_DATA_COLLECTION environment variable warning</pre></td></tr>

<tr class="ERROR"><td class="date">2026/01/14 07:17:37,400 UTC</td><td class="level">[ERROR]</td><td class="hierarchy">Thread: &lt;MainThread&gt;
test_module.test_function -- 123</td><td class="message"><pre class="ERROR">Connection failed: timeout</pre></td></tr>

</table>
</div>
</body>
</html>"#;

        let test_file = "real_test_log.html";
        std::fs::write(test_file, html_content).expect("Failed to write test file");

        let result = HtmlLogParser::parse_file(test_file, "test_session", 0);

        // Clean up
        let _ = std::fs::remove_file(test_file);

        assert!(result.is_ok());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 4);

        // Check first entry
        assert_eq!(entries[0].level, "[INFO]");
        assert_eq!(entries[0].timestamp, "2026/01/14 07:17:37,370 UTC");
        assert!(entries[0].message.contains("Package version info"));

        // Check second entry
        assert_eq!(entries[1].level, "[DEBUG]");
        assert_eq!(entries[1].timestamp, "2026/01/14 07:17:37,374 UTC");

        // Check third entry
        assert_eq!(entries[2].level, "[WARNING]");
        assert!(entries[2].message.contains("PYTEST_DATA_COLLECTION"));

        // Check fourth entry
        assert_eq!(entries[3].level, "[ERROR]");
        assert!(entries[3].message.contains("Connection failed"));
    }
}