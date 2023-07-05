use clap::{arg, Command};

fn main() {
    //TODO replace with args that matches rules_oci
    // mutate "base" --tag oci:registry/{ctx.attr.name}
    // --platform=
    // --append={layer}
    // --entrypoint ctx.attr.entrypoint joined with ,
    // --cmd  ctx.attr.cmd joined with ,
    // --user=ctx.attr.user
    // --workdir=ctx.attr.workdir
    // --env-file=ctx.attr.envfile
    // --labels-file=ctx.attr.labels
    // --annotations-file=ctx.attr.annotations
    // --output=
    let matches = Command::new("podman_bzl")
        .version("1.0")
        .author("mla")
        .about("Creates a podman oci_dir compatible storage while avoiding copying input layers")
        .arg(arg!([base] "Base of the container"))
        .arg(arg!(--tag <VALUE>).required(true))
        .arg(arg!(--output <VALUE>).required(true))
        .get_matches();

    let _base = matches.get_one::<String>("base");
    let _oci_dir = matches.get_one::<String>("output");
    // println!("Hello {}!", args.oci_dir);
    // let oci_dir = liboci::make_oci_dir(&args.oci_dir).expect("Failed");
    // let layers = liboci::make_layers_from_tars(args.tars).expect("Failed to make tars");
    // layers.iter().for_each(|l| println!("Layer {:?}", l));
    //let image = liboci::make_image(args.envs,args.entrypoint,layers).expect("Failed to make image")
}
