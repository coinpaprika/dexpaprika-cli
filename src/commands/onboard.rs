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
    println!("  Optional API key:");
    println!("    Not needed. The CLI works keyless and always will.");
    println!("    A free key raises the monthly credit allowance; it does not");
    println!("    raise the per-minute limit, which is the same on both free tiers.");
    println!("      dexpaprika-cli config set-key api_YOUR_KEY   # validates, then stores it");
    println!("      dexpaprika-cli config show                   # what the API makes of it");
    println!("    Or set DEXPAPRIKA_API_KEY. Paste the key on its own: no Bearer prefix.");
    println!();
    println!("  Good to know:");
    println!("    Free and paid plans are available;");
    println!("    see https://dexpaprika.com/api/pricing for the current quotas.");
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
