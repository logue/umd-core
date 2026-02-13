use umd::{parse, parse_with_frontmatter};

fn main() {
    println!("=== Test 1: YAML Frontmatter ===");
    let input1 = r#"---
title: My Document
author: John Doe
date: 2024-01-23
tags:
  - rust
  - wiki
---

# Welcome

This is a document with **YAML** frontmatter.
"#;

    let result = parse_with_frontmatter(input1);
    println!("Frontmatter present: {}", result.frontmatter.is_some());
    if let Some(fm) = &result.frontmatter {
        println!("Format: {:?}", fm.format);
        println!("Content:\n{}", fm.content);
    }
    println!("HTML output:\n{}", result.html);
    println!();

    println!("=== Test 2: TOML Frontmatter ===");
    let input2 = r#"+++
title = "My Document"
author = "Jane Smith"
date = 2024-01-23
+++

# Hello World

This document uses **TOML** frontmatter.
"#;

    let result = parse_with_frontmatter(input2);
    println!("Frontmatter present: {}", result.frontmatter.is_some());
    if let Some(fm) = &result.frontmatter {
        println!("Format: {:?}", fm.format);
        println!("Content:\n{}", fm.content);
    }
    println!("HTML output:\n{}", result.html);
    println!();

    println!("=== Test 3: No Frontmatter ===");
    let input3 = "# Simple Document\n\nJust plain content without frontmatter.";
    let result = parse_with_frontmatter(input3);
    println!("Frontmatter present: {}", result.frontmatter.is_some());
    println!("HTML output:\n{}", result.html);
    println!();

    println!("=== Test 4: parse() function (ignores frontmatter) ===");
    let html = parse(input1);
    println!("Using parse():\n{}", html);
}
