use umd::parse;

#[test]
fn test_semantic_html_integration() {
    // Test dfn
    let html = parse("&dfn{API};");
    println!("DFN output: {}", html);
    assert!(html.contains("<dfn>API</dfn>"), "Expected dfn tag, got: {}", html);
    
    // Test kbd
    let html = parse("&kbd{Ctrl};");
    println!("KBD output: {}", html);
    assert!(html.contains("<kbd>Ctrl</kbd>"), "Expected kbd tag, got: {}", html);
    
    // Test existing color (should work)
    let html = parse("&color(red){text};");
    println!("COLOR output: {}", html);
    assert!(html.contains("color: red"), "Expected color style, got: {}", html);
}
