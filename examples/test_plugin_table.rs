use umd::parse;

fn main() {
    println!("=== Table Plugin Examples ===\n");

    // Example 1: Basic table with striped and hover
    let basic = r#"@table(striped,hover){{
| Header1 | Header2 | Header3 |
|---------|---------|---------|
| Cell1   | Cell2   | Cell3   |
| Cell4   | Cell5   | Cell6   |
}}"#;
    println!("Example 1: Basic table with Bootstrap classes");
    println!("Input:\n{}\n", basic);
    println!("Output:\n{}\n", parse(basic));

    // Example 2: Responsive table
    let responsive = r#"@table(responsive){{
| Column1 | Column2 | Column3 | Column4 | Column5 |
|---------|---------|---------|---------|---------|
| Data1   | Data2   | Data3   | Data4   | Data5   |
}}"#;
    println!("Example 2: Responsive table");
    println!("Input:\n{}\n", responsive);
    println!("Output:\n{}\n", parse(responsive));

    // Example 3: Multiple tables (only first one styled)
    let multiple = r#"@table(hover){{
| Table1 | A |
|--------|---|
| Foo    | 1 |

| Table2 | B |
|--------|---|
| Bar    | 2 |
}}"#;
    println!("Example 3: Multiple tables (only first table styled)");
    println!("Input:\n{}\n", multiple);
    println!("Output:\n{}\n", parse(multiple));
    println!("Note: Only Table1 should have table-hover class\n");

    // Example 4: Combined options
    let combined = r#"@table(striped,hover,bordered,sm){{
| Compact | Table |
|---------|-------|
| Small   | Size  |
}}"#;
    println!("Example 4: Combined Bootstrap classes");
    println!("Input:\n{}\n", combined);
    println!("Output:\n{}\n", parse(combined));

    // Example 5: Dark mode table
    let dark = r#"@table(dark,striped){{
| Dark | Mode |
|------|------|
| Cell | Data |
}}"#;
    println!("Example 5: Dark mode table");
    println!("Input:\n{}\n", dark);
    println!("Output:\n{}\n", parse(dark));

    // Example 6: UMD table format with plugin
    let umd = r#"@table(hover){{
| ~Header1 | ~Header2 |h
| Cell1    | Cell2    |
| Cell3    | Cell4    |
}}"#;
    println!("Example 6: UMD table format with plugin");
    println!("Input:\n{}\n", umd);
    println!("Output:\n{}\n", parse(umd));

    // Example 7: Borderless table
    let borderless = r#"@table(borderless){{
| Clean | Look |
|-------|------|
| No    | Borders |
}}"#;
    println!("Example 7: Borderless table");
    println!("Input:\n{}\n", borderless);
    println!("Output:\n{}\n", parse(borderless));

    // Example 8: Non-nested approach (recommended)
    let separate = r#"@table(hover){{
| Table 1 | Data |
|---------|------|
| Foo     | Bar  |
}}

@table(striped){{
| Table 2 | Data |
|---------|------|
| Alpha   | Beta |
}}"#;
    println!("Example 8: Separate table plugins (recommended)");
    println!("Input:\n{}\n", separate);
    println!("Output:\n{}\n", parse(separate));
    println!("Note: This is the recommended way to style multiple tables\n");
}
