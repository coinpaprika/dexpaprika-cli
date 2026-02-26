use anyhow::Result;

pub fn execute() -> Result<()> {
    println!("┌──────────────────────────────────────────────────────────┐");
    println!("│  Welcome to dexpaprika-cli!                             │");
    println!("│  Free DEX data from your terminal                       │");
    println!("└──────────────────────────────────────────────────────────┘");
    println!();
    println!("  No API key needed. No rate limits. Completely free.");
    println!();
    println!("  Quick start:");
    println!("    dexpaprika-cli pools ethereum             # top pools on Ethereum");
    println!("    dexpaprika-cli token ethereum 0xc02a...   # token details");
    println!("    dexpaprika-cli stream ethereum 0xc02a...  # real-time prices");
    println!();
    println!("  Explore:");
    println!("    dexpaprika-cli networks                   # all supported chains");
    println!("    dexpaprika-cli stats                      # ecosystem overview");
    println!("    dexpaprika-cli search uniswap             # search everything");
    println!();
    println!("  Links:");
    println!("    API docs:  https://api.dexpaprika.com");
    println!("    Docs:      https://docs.dexpaprika.com");
    println!("    GitHub:    https://github.com/coinpaprika/dexpaprika-cli");
    println!();

    Ok(())
}
