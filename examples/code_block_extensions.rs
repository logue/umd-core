//! Example: Code block syntax highlighting and Mermaid diagram support
//!
//! This example demonstrates the new code block extensions for UMD:
//! - Syntax highlighting class generation
//! - File name support with <figcaption>
//! - Mermaid diagram rendering setup
//!
//! Run with: cargo run --example code_block_extensions

use umd::parse;

fn main() {
    println!("=== Code Block Extensions Example ===\n");

    // Example 1: Basic syntax highlighting
    println!("Example 1: Rust code with syntax highlighting");
    let rust_code = r#"
```rust
fn fibonacci(n: u32) -> u32 {
    match n {
        0 | 1 => n,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}
```
"#;

    let html = parse(rust_code);
    println!("Input:\n{}", rust_code);
    println!("Output:\n{}\n", html);

    // Example 2: Code block with filename
    println!("Example 2: Code with filename metadata");
    let python_code = r#"
```python
# @filename: data_processor.py
def process_data(data):
    """Process incoming data."""
    filtered = [x for x in data if x > 0]
    return sum(filtered) / len(filtered)
```
"#;

    let html = parse(python_code);
    println!("Input:\n{}", python_code);
    println!("Output:\n{}\n", html);

    // Example 3: Mermaid diagram
    println!("Example 3: Mermaid diagram (flow chart)");
    let mermaid_flowchart = r#"
```mermaid
graph TD
    A[User visits website] --> B{Logged in?}
    B -->|Yes| C[Show dashboard]
    B -->|No| D[Show login page]
    D --> E[User enters credentials]
    E --> F{Valid?}
    F -->|Yes| C
    F -->|No| E
    C --> G[Display user data]
```
"#;

    let html = parse(mermaid_flowchart);
    println!("Input:\n{}", mermaid_flowchart);
    println!("Output:\n{}\n", html);

    // Example 4: Mermaid sequence diagram
    println!("Example 4: Mermaid diagram (sequence)");
    let mermaid_sequence = r#"
```mermaid
sequenceDiagram
    participant Client
    participant Server
    participant Database
    
    Client->>Server: Request data
    Note over Server: Process request
    Server->>Database: Query records
    Database-->>Server: Return results
    Server-->>Client: Send JSON response
```
"#;

    let html = parse(mermaid_sequence);
    println!("Input:\n{}", mermaid_sequence);
    println!("Output:\n{}\n", html);

    // Example 5: Multiple code blocks
    println!("Example 5: Multiple code blocks (JavaScript and JSON)");
    let multi_code = r#"
```javascript
// @filename: fetch-config.js
const config = {
  apiUrl: 'https://api.example.com',
  timeout: 5000
};

async function fetchData(endpoint) {
  const response = await fetch(config.apiUrl + endpoint, {
    timeout: config.timeout
  });
  return response.json();
}
```

```json
{
  "name": "example-app",
  "version": "1.0.0",
  "description": "Example application"
}
```
"#;

    let html = parse(multi_code);
    println!("Input:\n{}", multi_code);
    println!("Output:\n{}\n", html);

    // Summary
    println!("\n=== Summary ===");
    println!("✅ Code blocks now support:");
    println!("  • Language detection and class generation");
    println!("  • File name metadata (@filename: comment)");
    println!("  • Mermaid diagram detection and wrapping");
    println!("  • Bootstrap CSS variable integration");
    println!("\n📋 Supported languages:");
    println!("  rust, python, javascript, typescript, html, css, sql, and more");
    println!("\n🎨 Mermaid diagram types:");
    println!("  • Flowcharts (graph)");
    println!("  • Sequence diagrams");
    println!("  • Class diagrams");
    println!("  • State diagrams");
    println!("  • ER diagrams");
    println!("  • Gantt charts");
    println!("  • Pie charts");
    println!("\n📖 Frontend Integration:");
    println!("  • Mermaid.js CDN for rendering");
    println!("  • Highlight.js or Prism.js for code highlighting");
    println!("  • Bootstrap CSS variables for theming");
}
