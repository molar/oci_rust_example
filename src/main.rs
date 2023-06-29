use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The name of the oci directory
    #[arg(short, long)]
    oci_dir: String,

    #[arg(required = true)]
    tars: Vec<PathBuf>,
}
use liboci;

fn main() {
    let args = Args::parse();
    //TODO replace with args that matches rules_oci

    println!("Hello {}!", args.oci_dir);
    let oci_dir = liboci::make_oci_dir(&args.oci_dir).expect("Failed");
    let layers = liboci::make_layers_from_tars(args.tars).expect("Failed to make tars");
    layers.iter().for_each(|l| println!("Layer {:?}", l));
    //let image = liboci::make_image(args.envs,args.entrypoint,layers).expect("Failed to make image")
}
