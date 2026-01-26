use lukiwiki_parser::parse;

fn main() {
    println!("=== Semantic HTML Elements Test ===\n");

    let test_cases = vec![
        // Ruby (Furigana)
        ("Ruby", "&ruby(Ashita){明日};"),
        // Semantic elements
        ("Definition", "&dfn{API};"),
        ("Keyboard", "Press &kbd{Ctrl+C}; to copy"),
        ("Sample output", "Command returns &samp{OK};"),
        ("Variable", "The value of &var{x}; is 10"),
        ("Citation", "&cite{The Great Gatsby};"),
        ("Quote", "He said &q{Hello};"),
        ("Small text", "&small{Fine print};"),
        ("Underline", "&u{Non-textual annotation};"),
        // Elements with attributes
        ("Time", "&time(2026-01-26){Today};"),
        ("Data", "Product &data(SKU-123){Widget}; available"),
        // Bidirectional text
        ("BDI", "&bdi{مرحبا}; means hello"),
        ("BDO", "&bdo(rtl){This flows right-to-left};"),
        // Word break
        (
            "WBR",
            "Super&wbr;cali&wbr;fragi&wbr;listic&wbr;expi&wbr;ali&wbr;docious",
        ),
        // Complex example
        (
            "Complex",
            "The &dfn{API}; requires &kbd{Enter}; to submit. \
          See &cite{Manual}; for &var{params};. \
          Current time: &time(2026-01-26T10:00){10:00 AM};",
        ),
    ];

    for (label, input) in test_cases {
        println!("Test: {}", label);
        println!("Input:  {}", input);
        let html = parse(input);
        println!("Output: {}\n", html.trim());
    }

    // Combined example with multiple features
    println!("=== Combined Features Example ===\n");
    let combined = r#"
# Semantic HTML Demo {#semantic-demo}

The &dfn{HyperText Markup Language}; (&abbr(HTML){HyperText Markup Language};) 
allows rich formatting:

- Press &kbd{F5}; to refresh
- Variable &var{x}; equals &data(42){forty-two};
- Published on &time(2026-01-26){January 26, 2026};
- Quote: &q{To be or not to be};
- Book: &cite{Alice in Wonderland};

Japanese: &ruby(にほんご){日本語}; with furigana.

Break long&wbr;words&wbr;elegantly.
"#;

    println!("Input:");
    println!("{}", combined);
    println!("\nOutput:");
    let html = parse(combined);
    println!("{}", html.trim());
}
