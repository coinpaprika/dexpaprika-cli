use anyhow::Result;

pub fn execute() -> Result<()> {
    println!("┌──────────────────────────────────────────────────────────┐");
    println!("│  Welcome to dexpaprika-cli!                             │");
    println!("│  Free DEX data from your terminal                       │");
    println!("└──────────────────────────────────────────────────────────┘");
    println!();
    println!("  Free tier: no API key, no credit card, just start querying.");
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
    println!("  Good to know:");
    println!("    No API key needed to start. Free and paid plans are available;");
    println!("    see https://dexpaprika.com/pricing for the current quotas.");
    println!("    Streaming is metered like REST: one delivered update = one credit.");
    println!("    Commercial use requires attribution with a do-follow link.");
    println!("    Run dexpaprika-cli attribution for copy-paste snippets.");
    println!();
    println!("  Need higher limits, SLA, or enterprise support?");
    println!("    support@coinpaprika.com");
    println!();
    println!("  Links:");
    println!("    API docs:  https://api.dexpaprika.com");
    println!("    Docs:      https://docs.dexpaprika.com");
    println!("    GitHub:    https://github.com/coinpaprika/dexpaprika-cli");
    println!();

    Ok(())
}
