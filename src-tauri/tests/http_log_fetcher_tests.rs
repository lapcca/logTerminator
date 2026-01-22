use logterminator_lib::http_log_fetcher::HttpLogFetcher;

#[test]
fn test_parse_apache_directory_listing() {
    let html = r#"
<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 3.2 Final//EN">
<html>
<head>
  <title>Index of /logs</title>
</head>
<body>
<h1>Index of /logs</h1>
<table>
  <tr><th valign="top"><img src="/icons/blank.gif" alt="[ICO]"></th><th><a href="?C=N;O=D">Name</a></th><th><a href="?C=M;O=A">Last modified</a></th><th><a href="?C=S;O=A">Size</a></th></tr>
  <tr><th colspan="4"><hr></th></tr>
  <tr><td valign="top"><img src="/icons/back.gif" alt="[PARENTDIR]"></td><td><a href="../">Parent Directory</a></td><td>&nbsp;</td><td align="right">  - </td></tr>
  <tr><td valign="top"><img src="/icons/unknown.gif" alt="[   ]"></td><td><a href="TestEnableTcpdump_ID_1---0.html">TestEnableTcpdump_ID_1---0.html</a></td><td align="right">2025-01-15 10:30  </td><td align="right"> 12K</td></tr>
  <tr><td valign="top"><img src="/icons/unknown.gif" alt="[   ]"></td><td><a href="TestEnableTcpdump_ID_1---1.html">TestEnableTcpdump_ID_1---1.html</a></td><td align="right">2025-01-15 10:31  </td><td align="right"> 15K</td></tr>
  <tr><td valign="top"><img src="/icons/folder.gif" alt="[DIR]"></td><td><a href="subdir/">subdir/</a></td><td align="right">2025-01-15 10:32  </td><td align="right">  - </td></tr>
</table>
</body></html>
    "#;

    let urls = HttpLogFetcher::parse_directory_listing(html, "http://example.com/logs/").unwrap();
    assert_eq!(urls.len(), 2);
    assert!(urls.contains(&"http://example.com/logs/TestEnableTcpdump_ID_1---0.html".to_string()));
    assert!(urls.contains(&"http://example.com/logs/TestEnableTcpdump_ID_1---1.html".to_string()));
}
