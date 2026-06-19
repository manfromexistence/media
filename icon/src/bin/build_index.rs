use dx_icons::builder::IndexBuilder;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let data_dir = PathBuf::from("data");
    let output_dir = PathBuf::from("index");

    IndexBuilder::build_from_dir(&data_dir, &output_dir)?;

    Ok(())
}
