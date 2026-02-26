use tabled::{Table, Tabled};
use tabled::settings::Style;

use crate::commands::networks::{Network, Dex};
use crate::output::{format_usd, print_dexpaprika_footer};

#[derive(Tabled)]
struct NetworkRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
}

pub fn print_networks_table(networks: &[Network]) {
    let rows: Vec<NetworkRow> = networks.iter().map(|n| NetworkRow {
        id: n.id.clone(),
        name: n.display_name.clone().unwrap_or_else(|| n.id.clone()),
    }).collect();

    let table = Table::new(rows).with(Style::rounded()).to_string();
    println!("{table}");
    print_dexpaprika_footer();
}

#[derive(Tabled)]
struct DexRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Protocol")]
    protocol: String,
    #[tabled(rename = "Pools")]
    pools: String,
    #[tabled(rename = "Volume (USD)")]
    volume: String,
}

pub fn print_dexes_table(dexes: &[Dex]) {
    let rows: Vec<DexRow> = dexes.iter().map(|d| DexRow {
        id: d.dex_id.clone().unwrap_or_else(|| "—".into()),
        name: d.dex_name.clone().unwrap_or_else(|| "—".into()),
        protocol: d.protocol.clone().unwrap_or_else(|| "—".into()),
        pools: d.pools_count.map(|p| p.to_string()).unwrap_or_else(|| "—".into()),
        volume: d.volume_usd.map(format_usd).unwrap_or_else(|| "—".into()),
    }).collect();

    let table = Table::new(rows).with(Style::rounded()).to_string();
    println!("{table}");
    print_dexpaprika_footer();
}
