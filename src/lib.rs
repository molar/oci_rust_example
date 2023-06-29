use oci_spec::image::{
    Descriptor, DescriptorBuilder, ImageConfigurationBuilder, ImageIndex, ImageIndexBuilder,
    ImageManifest, ImageManifestBuilder, MediaType, SCHEMA_VERSION,
};
use std::iter::Map;

use sha2::{Digest, Sha256};
use std::fs;
use std::io::prelude::*;
use std::path::PathBuf;

fn sha256_digest<R: Read>(mut reader: R) -> std::io::Result<(String, usize)> {
    let mut buffer = [0; 1024];
    let mut hasher = Sha256::new();
    let mut len: usize = 0;

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
        len = len + count;
    }

    Ok((format!("sha256:{:x}", hasher.finalize()), len))
}

pub fn make_oci_dir(name: &str) -> std::io::Result<()> {
    fs::create_dir_all(name)?;
    let oci_blobs_dir: PathBuf = [name, "blobs", "sha256"].iter().collect();
    fs::create_dir_all(oci_blobs_dir)?;
    let oci_layout_file: PathBuf = [name, "oci-layout"].iter().collect();
    let mut file = fs::File::create(oci_layout_file)?;
    file.write_all(b"{\"imageLayoutVersion\":\"1.0.0\"}");
    Ok(())
}

pub fn make_layers_from_tars(tars: Vec<PathBuf>) -> std::io::Result<Vec<Descriptor>> {
    Ok(tars
        .iter()
        .map(|t| {
            let file = fs::File::open(t).expect("File should really be there");
            let (digest, size) = sha256_digest(file).expect("calculating hash");
            DescriptorBuilder::default()
                .media_type(MediaType::ImageLayer)
                .size(i64::try_from(size).expect("Getting size"))
                .digest(digest)
                .build()
                .expect("Adding Layer")
        })
        .collect())
}

pub fb add_image_config()
pub fn add_image(env: Map<String, String>, layers: Vec<Descriptor>) -> std::io::Result<()> {
    Ok(())
}
