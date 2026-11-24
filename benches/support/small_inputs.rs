//! Shared inputs for small-document timing and allocation counts.

pub(crate) const SAMPLES: &[(&str, &str)] = &[
    ("tiny_ok", r#"{"ok":true}"#),
    (
        "ticker",
        r#"{"e":"24hrTicker","E":1732450000123,"s":"BTCUSDT","p":"-120.50","P":"-0.13","c":"94210.10","Q":"0.012","b":"94210.00","a":"94210.10","v":"18234.551","q":"1717171717.12"}"#,
    ),
    (
        "escaped",
        r#"{"msg":"line1\nline2 \"quoted\" \u00e9\ud83d\ude00","path":"C:\\temp\\x"}"#,
    ),
    (
        "nested",
        r#"{"meta":{"name":"json","versions":[1,2,3,4]},"users":[{"id":1,"name":"a","flags":[true,false,null]},{"id":2,"name":"b","flags":[false,true,null]}],"config":{"retry":3,"timeout":1.25,"enabled":true}}"#,
    ),
];
