use anyhow::Result;
use serde::Deserialize;
use serde_yaml::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "yaml-to-fstab")]
struct Opt {
    #[structopt(short, long)]
    conf: String,
}

#[derive(Deserialize, Debug)]
struct Input {
    fstab: HashMap<String, Mounts>,
}

#[derive(Deserialize, Debug)]
struct Mounts {
    mount: String,
    export: Option<String>,
    #[serde(rename = "type")]
    fs_type: String,
    #[serde(rename = "root-reserve")]
    root_reserve: Option<String>,
    #[serde(default)]
    options: Vec<String>,
}

fn main() -> Result<()> {
    //let opt = Opt::from_args();

    let file = File::open("example.yml")?;
    let reader = BufReader::new(file);

    let m: Input = serde_yaml::from_reader(reader)?;

    println!("{:#?}", m);

    Ok(())
}
