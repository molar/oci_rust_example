use oci_spec::image::{
    Descriptor, 
    DescriptorBuilder, 
    ImageManifest, 
    ImageManifestBuilder, 
    ImageIndex,
    ImageIndexBuilder,
    MediaType, 
    SCHEMA_VERSION
};

use std::path::PathBuf;
use std::fs;
use std::io::prelude::*;
use sha2::{Sha256,  Digest};

fn sha256_digest<R: Read>(mut reader: R) -> std::io::Result<(String,usize)> {
    let mut buffer = [0; 1024];
    let mut hasher = Sha256::new();
    let mut len : usize = 0;

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
        len = len + count;
    }

    Ok((format!("sha256:{:x}",hasher.finalize()),len))
}

fn make_oci_dir(name: &str) -> std::io::Result<()> { 
    fs::create_dir_all(name)?;
    let oci_blobs_dir: PathBuf = [name, "blobs","sha256"].iter().collect();
    fs::create_dir_all(oci_blobs_dir)?;
    let oci_layout_file: PathBuf = [name, "oci-layout"].iter().collect();
    let mut file = fs::File::create(oci_layout_file)?;
    file.write_all(b"{\"imageLayoutVersion\":\"1.0.0\"}");
    Ok(())
}


fn make_layers_from_tars(tars: Vec<PathBuf>) -> std::io::Result<Vec<Descriptor>>{

    Ok(tars.iter().map(|t|{
        let file = fs::File::open(t).expect("File should really be there");
        let (digest,size) = sha256_digest(file).expect("calculating hash");
        DescriptorBuilder::default()
        .media_type(MediaType::ImageLayer)
        .size(i64::try_from(size).expect("Getting size"))
            .digest(digest).build().expect("Adding Layer")
        
    }).collect())
}

fn add_image(layers: Vec<Descriptor>) -> std::io::Result<()> {
    // for each layer()
    Ok(())
}

fn main() {
    println!("Hello, world!");
    make_oci_dir("test_image");
let config = DescriptorBuilder::default()
            .media_type(MediaType::ImageConfig)
            .size(7023)
            .digest("sha256:b5b2b2c507a0944348e0303114d8d93aaaa081732b86451d9bce1f432a537bc7")
            .build()
            .expect("build config descriptor");

let layers: Vec<Descriptor> = [
    (
        32654,
        "sha256:9834876dcfb05cb167a5c24953eba58c4ac89b1adf57f28f2f9d09af107ee8f0",
    ),
    (
        16724,
        "sha256:3c3a4604a545cdc127456d94e421cd355bca5b528f4a9c1905b15da2eb4a4c6b",
    ),
    (
        73109,
        "sha256:ec4b8955958665577945c89419d1af06b5f7636b4ac3da7f12184802ad867736",
    ),
]
    .iter()
    .map(|l| {
    DescriptorBuilder::default()
        .media_type(MediaType::ImageLayer)
        .size(l.0)
        .digest(l.1.to_owned())
        .build()
        .expect("build layer")
    })
    .collect();

let image_manifest = ImageManifestBuilder::default()
    .schema_version(SCHEMA_VERSION)
    .config(config)
    .layers(layers)
    .build()
    .expect("build image manifest");
image_manifest.to_file_pretty("test_image/index.json");
println!("Got {}",image_manifest.to_string_pretty().unwrap());
    let indexes :Vec<Descriptor> = [image_manifest].iter()
    .map(|l| {
            let data = l.to_string_pretty().unwrap();
            let size = data.len();
            let mut hasher = Sha256::new();
            hasher.update(data);
            DescriptorBuilder::default()
            .media_type(MediaType::ImageManifest)
                .size(i64::try_from(size).unwrap())
                .digest(format!("sha256:{:x}",hasher.finalize())).build().unwrap()
        }).collect();
let image_index= ImageIndexBuilder::default()
        .schema_version(2u32)
        .manifests(indexes)
        .build();
}
