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
            println!("  HTML:");
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
            println!("  Free forever. No API key. No limits.");
            println!("  API: api.dexpaprika.com");
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
