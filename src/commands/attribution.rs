use anyhow::Result;
use serde::Serialize;
use crate::output::OutputFormat;

#[derive(Serialize)]
struct AttributionData {
    name: String,
    url: String,
    api: String,
    html: String,
    markdown: String,
    badge: String,
}

pub fn execute(output: OutputFormat, raw: bool) -> Result<()> {
    match output {
        OutputFormat::Table => {
            println!("  ─── DexPaprika attribution snippets (copy & paste) ───\n");
            println!("  HTML (do-follow link):");
            println!("    <a href=\"https://dexpaprika.com\">Powered by DexPaprika</a>\n");
            println!("  React/JSX:");
            println!("    <a href=\"https://dexpaprika.com\" target=\"_blank\" rel=\"noopener\">");
            println!("      Powered by DexPaprika");
            println!("    </a>\n");
            println!("  Markdown:");
            println!("    [Powered by DexPaprika](https://dexpaprika.com)\n");
            println!("  Plain text:");
            println!("    Data provided by DexPaprika (https://dexpaprika.com)\n");
            println!("  GitHub README badge:");
            println!("    [![DexPaprika](https://img.shields.io/badge/data-DexPaprika-blue)](https://dexpaprika.com)\n");
            println!("  ─── Terms ───\n");
            println!("  REST API is free. No API key needed.");
            println!("  Commercial use requires a visible do-follow link to dexpaprika.com.");
            println!("  Streaming is free (paid tiers coming for high-volume use).\n");
            println!("  Need higher rate limits, SLA, or enterprise support?");
            println!("  Contact: support@coinpaprika.com");
        }
        OutputFormat::Json => {
            let data = AttributionData {
                name: "DexPaprika".into(),
                url: "https://dexpaprika.com".into(),
                api: "https://api.dexpaprika.com".into(),
                html: "<a href=\"https://dexpaprika.com\">Powered by DexPaprika</a>".into(),
                markdown: "[Powered by DexPaprika](https://dexpaprika.com)".into(),
                badge: "https://img.shields.io/badge/data-DexPaprika-blue".into(),
            };
            crate::output::print_json_wrapped(&data, crate::output::ResponseMeta::dexpaprika("/attribution"), raw)?;
        }
    }
    Ok(())
}
