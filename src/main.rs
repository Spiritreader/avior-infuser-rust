mod db;
mod cfg;
use std::error::Error;

const CFG_PATH: &str = "config.json";

fn main() -> Result<(), Box<dyn Error>> {
    let config = cfg::read(CFG_PATH)?;
    let client = db::connect(&config)?;
    let result = db::get_clients(client, &config.db_name)?;
    println!("{:?}", result);
    Ok(())
}
