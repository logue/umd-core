use umd::parse;

fn main() {
    // Test %%text%% → <s>text</s>
    let input1 = "This is %%LukiWiki strikethrough%% text.";
    println!("Input 1: {}", input1);
    let output1 = parse(input1);
    println!("Output 1: {}", output1);
    println!();

    // Test ~~text~~ → <del>text</del>
    let input2 = "This is ~~GFM strikethrough~~ text.";
    println!("Input 2: {}", input2);
    let output2 = parse(input2);
    println!("Output 2: {}", output2);
    println!();

    // Test both together
    let input3 = "%%LukiWiki%% and ~~GFM~~ strikethrough.";
    println!("Input 3: {}", input3);
    let output3 = parse(input3);
    println!("Output 3: {}", output3);
    println!();

    // Validate
    println!("Validation:");
    if output1.contains("<s>LukiWiki strikethrough</s>") {
        println!("✓ %%text%% is converted to <s>text</s>");
    } else {
        println!("✗ %%text%% conversion failed");
    }

    if output2.contains("<del>GFM strikethrough</del>") {
        println!("✓ ~~text~~ is converted to <del>text</del>");
    } else {
        println!("✗ ~~text~~ conversion failed");
    }

    if output3.contains("<s>LukiWiki</s>") && output3.contains("<del>GFM</del>") {
        println!("✓ Both syntax can be used together");
    } else {
        println!("✗ Mixed syntax conversion failed");
    }
}
