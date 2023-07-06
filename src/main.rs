use std::path::PathBuf;

use clap::{arg, ArgAction, Command};
use liboci::{get_descriptor, make_layers_from_tars, DescriptorLike};
use oci_spec::image::{Arch, ImageConfiguration, ImageManifest};

// mutate "base" --tag oci:registry/{ctx.attr.name}
// --platform=
// --append={layer}
// --output=
//
fn main() {
    // --entrypoint ctx.attr.entrypoint joined with ,
    // --cmd  ctx.attr.cmd joined with ,
    // --user=ctx.attr.user
    // --workdir=ctx.attr.workdir
    // --env-file=ctx.attr.envfile
    // --labels-file=ctx.attr.labels
    // --annotations-file=ctx.attr.annotations
    let main_matches = Command::new("podman_bzl").subcommand(
        Command::new("mutate")
            .version("1.0")
            .author("mla")
            .about(
                "Creates a podman oci_dir compatible storage while avoiding copying input layers",
            )
            .arg(arg!([base] "Base of the container"))
            .arg(arg!(--tag <VALUE>).required(true))
            .arg(arg!(--output <VALUE>).required(true))
            .arg(arg!(--append <VALUE>).action(ArgAction::Append))
            .arg(arg!(--platform <VALUE>))
    ).get_matches();
    match main_matches.subcommand() {
        Some(("mutate", matches)) => {
            let _base = matches.get_one::<String>("base").unwrap();
            let _oci_dir = matches.get_one::<String>("output").unwrap();
            let _tag = matches.get_one::<String>("tag").unwrap();
            let _layers = matches
                .get_many::<String>("append")
                .unwrap_or_default()
                .map(|v| v.as_str())
                .collect::<Vec<_>>();
            let _platform = matches.get_one::<String>("platform");

            println!("Making oci dir at {}", _oci_dir);
            if let Ok(oci_dir) = liboci::make_oci_dir(_oci_dir) {
                println!("Adding base from {}", _base);
                let index = oci_dir.add_base_oci_dir(&PathBuf::from(_base));
                let layers = _layers.iter().map(|m| PathBuf::from(m)).collect();
                println!("Adding layers  {:?}", layers);
                let tar_layers = make_layers_from_tars(layers).unwrap();

                let mut existing_image = ImageManifest::from_file(
                    oci_dir
                        .get_descriptor_file(index.manifests().first().unwrap())
                        .unwrap(),
                )
                .unwrap();

                tar_layers.iter().for_each(|m| {
                    oci_dir.link_descriptor(&m.0, &m.1);
                    existing_image.layers_mut().push(m.0.clone());
                });

                // read the image config and mutate according to flags
                let config_fd = oci_dir
                    .get_descriptor_file(existing_image.config())
                    .unwrap();
                let mut image_config = ImageConfiguration::from_file(config_fd).unwrap();
                let new_config = image_config.config().clone().unwrap();
                if let Some(platform) = _platform {
                    let arch = match platform.as_str() {
                        "x86_64" => Arch::Amd64,
                        "aarch64" => Arch::ARM64,
                        _ => Arch::Amd64,
                    };
                    println!("Writing platform {:?}", arch);
                    image_config.set_architecture(arch);
                }
                //new_config.env()

                image_config.set_config(Some(new_config));

                //write the new config
                let (descriptor, datablob) =
                    get_descriptor(&DescriptorLike::Config { 0: &image_config });
                oci_dir.write_descriptor(&descriptor, datablob);

                existing_image.set_config(descriptor);

                //read and mutate the busybox image
                oci_dir.set_image_tag(&existing_image, _tag);
            }
        }
        _ => println!("Dont know what to do"),
    }
}
