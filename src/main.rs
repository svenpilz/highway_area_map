use clap::Parser;
use highway_area_map::*;
use std::fs;

#[derive(Parser)]
struct Args {
    export_file_name: String,
}

fn main() {
    let args = Args::parse();
    HighwayAreaMap::new(fs::read(args.export_file_name).ok()).run();
}
