//! Example: Code block metadata extensions
//!
//! This example demonstrates umd-core's code block extensions:
//! - Language class passthrough (no highlighting is done in core)
//! - File name support with <figcaption>
//! - `<figure class="umd-code-block">` wrapping, including for languages
//!   like `mermaid` that a host application renders itself (see
//!   docs/architecture.md's 3-layer deferred-rendering model)
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

    // Example 3: Mermaid code block (passed through, not rendered by core)
    println!("Example 3: Mermaid code block (host application renders this)");
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

    // Example 4: Another Mermaid code block (sequence diagram)
    println!("Example 4: Mermaid code block (sequence diagram, host-rendered)");
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
    println!("✅ umd-core's code block handling:");
    println!("  • Language class passthrough (`language-<lang>`), unmodified");
    println!("  • File name metadata (@filename: comment) → <figcaption>");
    println!("  • Every block wrapped in <figure class=\"umd-code-block\">");
    println!(
        "\n📖 Host application responsibility (Layer 2/3, see docs/architecture.md):"
    );
    println!("  • Syntax highlighting (e.g. highlight.js, Shiki, Prism.js)");
    println!("  • Mermaid diagram rendering (e.g. mermaid.js client-side,");
    println!("    or a native renderer at build time)");
    println!("  • Any other language-specific enhancement (e.g. GeoJSON maps)");
}
