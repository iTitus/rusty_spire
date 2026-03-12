use rusty_spire::save::history::RunHistory;
use std::fs::File;
use std::io::BufReader;

fn main() -> anyhow::Result<()> {
    read_history()?;
    Ok(())
}

#[allow(unused)]
fn read_history() -> anyhow::Result<()> {
    let sts2_data_dir = directories::BaseDirs::new()
        .ok_or_else(|| anyhow::anyhow!("cannot find sts2 dir"))?
        .data_dir()
        .join("SlayTheSpire2");
    let history_dir = sts2_data_dir.join("steam/76561198094479556/profile1/saves/history");
    let file = history_dir.join("1772886548.run");

    let run_history: RunHistory = serde_json::from_reader(BufReader::new(File::open(&file)?))?;
    println!("{run_history:?}");

    Ok(())
}
