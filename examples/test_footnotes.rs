use umd::parse_with_frontmatter;

fn main() {
    println!("=== Test 1: Simple Footnote ===");
    let input1 = r#"# Document with Footnotes

This is a sentence with a footnote[^1].

Another paragraph with another footnote[^2].

[^1]: This is the first footnote.
[^2]: This is the second footnote with **bold** text.
"#;

    let result = parse_with_frontmatter(input1);
    println!("Body HTML:");
    println!("{}", result.html);
    println!("\nFootnotes present: {}", result.footnotes.is_some());
    if let Some(footnotes) = &result.footnotes {
        println!("Footnotes HTML:");
        println!("{}", footnotes);
    }
    println!("\n{}\n", "=".repeat(60));

    println!("=== Test 2: With Frontmatter and Footnotes ===");
    let input2 = r#"---
title: Research Paper
author: John Doe
---

# Introduction

Recent studies[^study1] show interesting results.

## Methodology

We used the approach described by Smith[^smith2020].

[^study1]: Johnson et al. (2023). "New Findings in Rust Development"
[^smith2020]: Smith, J. (2020). *Research Methods*
"#;

    let result = parse_with_frontmatter(input2);
    println!("Frontmatter present: {}", result.frontmatter.is_some());
    if let Some(fm) = &result.frontmatter {
        println!("Title: {}", fm.content.lines().next().unwrap_or(""));
    }
    println!("\nBody HTML:");
    println!("{}", result.html);
    println!("\nFootnotes HTML:");
    if let Some(footnotes) = &result.footnotes {
        println!("{}", footnotes);
    }
    println!("\n{}\n", "=".repeat(60));

    println!("=== Test 3: No Footnotes ===");
    let input3 = "# Simple Document\n\nNo footnotes here.";
    let result = parse_with_frontmatter(input3);
    println!("Footnotes present: {}", result.footnotes.is_some());
    println!("Body HTML:");
    println!("{}", result.html);
}
