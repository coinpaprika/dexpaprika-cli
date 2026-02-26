use crate::commands::status::StatusResult;
use crate::output::{detail_field, print_detail_table};

pub fn print_status(result: &StatusResult) {
    let mut rows: Vec<[String; 2]> = Vec::new();
    detail_field!(rows, "DexPaprika API", format!("{} ({}ms)", result.dexpaprika.status, result.dexpaprika.response_time_ms));
    print_detail_table(rows);
}
