use std::path::PathBuf;

use clap::{arg, ArgAction, Command};
use liboci::{get_descriptor, make_layers_from_tars, DescriptorLike};
use oci_spec::image::{Arch, ImageConfiguration, ImageManifest};

mod utils;
// mutate "base" --tag oci:registry/{ctx.attr.name}
// --platform=
// --append={layer}
// --output=
// --env-file=ctx.attr.envfile
// --labels-file=ctx.attr.labels
// --annotations-file=ctx.attr.annotations
// --user=ctx.attr.user
// --entrypoint ctx.attr.entrypoint joined with ,
// --cmd  ctx.attr.cmd joined with ,
// --workdir=ctx.attr.workdir
//
fn main() {
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
            .arg(arg!(--"env-file" <VALUE>))
            .arg(arg!(--"annotations-file" <VALUE>))
            .arg(arg!(--"labels-file" <VALUE>))
            .arg(arg!(--user <VALUE>))
            .arg(arg!(--entrypoint <VALUE>))
            .arg(arg!(--cmd <VALUE>))
            .arg(arg!(--workdir <VALUE>))
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
            let _env_file = matches.get_one::<String>("env-file");
            let _annotations_file = matches.get_one::<String>("annotations-file");
            let _labels_file = matches.get_one::<String>("labels-file");

            println!("Making oci dir at {}", _oci_dir);
            if let Ok(oci_dir) = liboci::make_oci_dir(_oci_dir) {
                println!("Adding base from {}", _base);
                let index = oci_dir.add_base_oci_dir(&PathBuf::from(_base));
                let layers = _layers.iter().map(PathBuf::from).collect();
                println!("Adding layers  {:?}", layers);
                let tar_layers = make_layers_from_tars(layers).unwrap();

                let mut existing_image = ImageManifest::from_file(
                    oci_dir
                        .get_descriptor_file(index.manifests().first().unwrap())
                        .unwrap(),
                )
                .unwrap();

                tar_layers.iter().for_each(|m| {
                    oci_dir.link_descriptor(&m.0, &m.1).unwrap();
                    existing_image.layers_mut().push(m.0.clone());
                });

                // read the image config and mutate according to flags
                let config_fd = oci_dir
                    .get_descriptor_file(existing_image.config())
                    .unwrap();
                let mut image_config = ImageConfiguration::from_file(config_fd).unwrap();
                let mut new_config = image_config.config().clone().unwrap();
                if let Some(platform) = _platform {
                    let arch = match platform.as_str() {
                        "x86_64" => Arch::Amd64,
                        "amd64" => Arch::Amd64,
                        "aarch64" => Arch::ARM64,
                        str => panic!("Dont know arch {}", str),
                    };
                    println!("Writing platform {:?}", arch);
                    image_config.set_architecture(arch);
                }
                if let Some(env_file) = _env_file {
                    let env = utils::read_kv_file(&PathBuf::from(env_file)).unwrap();
                    let mut env_vec: Vec<String> = Vec::<String>::new();
                    if let Some(existing_env) = new_config.env() {
                        existing_env.iter().for_each(|e| env_vec.push(e.clone()));
                    }
                    env_vec.extend(env.iter().map(|m| format!("{}={}", m.0, m.1)));

                    new_config.set_env(Some(env_vec));
                }
                if let Some(labels_file) = _labels_file {
                    let labels = utils::read_kv_file(&PathBuf::from(labels_file)).unwrap();
                    if let Some(existing_labels) = new_config.labels_mut() {
                        existing_labels.extend(labels);
                    } else {
                        new_config.set_labels(Some(labels));
                    }
                }
                if let Some(annotations_file) = _annotations_file {
                    let annotations =
                        utils::read_kv_file(&PathBuf::from(annotations_file)).unwrap();
                    if let Some(image_annotations) = existing_image.annotations_mut() {
                        image_annotations.extend(annotations);
                    } else {
                        existing_image.set_annotations(Some(annotations));
                    }
                }

                if let Some(user) = matches.get_one::<String>("user") {
                    new_config.set_user(Some(user.clone()));
                }

                if let Some(entrypoint) = matches.get_one::<String>("entrypoint") {
                    let entry_point_split: Vec<String> =
                        entrypoint.split(',').map(String::from).collect();
                    new_config.set_entrypoint(Some(entry_point_split));
                }

                if let Some(cmd) = matches.get_one::<String>("cmd") {
                    let cmd_split: Vec<String> = cmd.split(',').map(String::from).collect();
                    new_config.set_cmd(Some(cmd_split));
                }

                if let Some(workdir) = matches.get_one::<String>("workdir") {
                    new_config.set_working_dir(Some(workdir.clone()));
                }

                image_config.set_config(Some(new_config));

                //write the new config
                let (descriptor, datablob) =
                    get_descriptor(&DescriptorLike::Config(&image_config)).unwrap();
                oci_dir.write_descriptor(&descriptor, datablob);

                existing_image.set_config(descriptor);

                //read and mutate the busybox image
                oci_dir.set_image(&existing_image);
            }
        }
        _ => println!("Dont know what to do"),
    }
}
