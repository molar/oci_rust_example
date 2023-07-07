use oci_spec::image::{
    ConfigBuilder, Descriptor, DescriptorBuilder, ImageConfiguration, ImageConfigurationBuilder,
    ImageIndex, ImageIndexBuilder, ImageManifest, ImageManifestBuilder, MediaType, SCHEMA_VERSION,
};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::prelude::*;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::{fs, result};

#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    OciSpecError(oci_spec::OciSpecError),
}

impl std::convert::From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IOError(err)
    }
}
impl std::convert::From<oci_spec::OciSpecError> for Error {
    fn from(err: oci_spec::OciSpecError) -> Self {
        Error::OciSpecError(err)
    }
}

pub type Result<T> = result::Result<T, Error>;

fn sha256_digest<R: Read>(mut reader: R) -> Result<(String, usize)> {
    let mut buffer = [0; 1024];
    let mut hasher = Sha256::new();
    let mut len: usize = 0;

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
        len += count;
    }

    Ok((format!("{:x}", hasher.finalize()), len))
}

pub enum DescriptorLike<'a> {
    Image(&'a ImageManifest),
    Config(&'a ImageConfiguration),
    ImageIndex(&'a ImageIndex),
}

fn get_descriptor_raw(t: MediaType, bytes: String) -> Result<(Descriptor, String)> {
    let (sha, size) = sha256_digest(bytes.as_bytes())?;

    Ok((
        Descriptor::new(t, size as i64, format!("sha256:{}", sha)),
        bytes,
    ))
}

pub fn get_descriptor(mig: &DescriptorLike) -> Result<(Descriptor, String)> {
    match *mig {
        DescriptorLike::Image(desc) => {
            let data_str = desc.to_string_pretty()?;
            get_descriptor_raw(MediaType::ImageManifest, data_str)
        }
        DescriptorLike::Config(desc) => {
            let data_str = desc.to_string_pretty()?;
            get_descriptor_raw(MediaType::ImageConfig, data_str)
        }
        DescriptorLike::ImageIndex(desc) => {
            let data_str = desc.to_string_pretty()?;
            get_descriptor_raw(MediaType::ImageIndex, data_str)
        }
    }
}
pub fn get_blob_from_digest(digest: &str) -> Option<&str> {
    digest.split(':').last()
}

pub fn get_file_from_descriptor(root_dir: &Path, desc: &Descriptor) -> Option<PathBuf> {
    Some(
        root_dir
            .join("blobs/sha256")
            .join(get_blob_from_digest(desc.digest())?),
    )
}

pub struct OciDir {
    pub base: PathBuf,
    pub blob_dir: PathBuf,
}

impl OciDir {
    pub fn link_descriptor(&self, desc: &Descriptor, realpath: &Path) -> Result<()> {
        if let Some(digest_name) = get_blob_from_digest(desc.digest()) {
            symlink(fs::canonicalize(realpath)?, self.blob_dir.join(digest_name))?;
        }
        Ok(())
    }

    pub fn get_descriptor_file(&self, desc: &Descriptor) -> Option<PathBuf> {
        get_file_from_descriptor(&self.base, desc)
    }

    pub fn write_descriptor(&self, desc: &Descriptor, data: String) {
        if let Some(digest_name) = desc.digest().split(':').last() {
            let mut file = fs::File::create(self.blob_dir.join(digest_name)).unwrap();
            file.write_all(data.as_bytes()).unwrap();
        }
    }

    pub fn add_base_oci_dir(&self, p: &Path) -> ImageIndex {
        let index = ImageIndex::from_file(p.join("index.json")).unwrap();
        index.to_file_pretty(self.base.join("index.json")).unwrap();

        let blobs_to_link: Vec<Descriptor> = index
            .manifests()
            .iter()
            .filter_map(|man| {
                if let Ok(image) =
                    ImageManifest::from_file(get_file_from_descriptor(p, man).unwrap())
                {
                    Some(image.layers().clone())
                } else {
                    None
                }
            })
            .flatten()
            .collect();
        for blob in blobs_to_link {
            let blob_name = get_blob_from_digest(blob.digest()).unwrap();
            let blob_path = p.join("blobs/sha256").join(blob_name);
            let src = fs::canonicalize(blob_path).unwrap();
            let dst = self.blob_dir.join(blob_name);
            symlink(src, dst).unwrap();
        }

        let blobs_to_copy: Vec<Descriptor> = index
            .manifests()
            .iter()
            .filter_map(|m| match m.media_type() {
                MediaType::ImageManifest => Some(vec![
                    m.clone(),
                    ImageManifest::from_file(get_file_from_descriptor(p, m).unwrap())
                        .unwrap()
                        .config()
                        .clone(),
                ]),
                _ => None,
            })
            .flatten()
            .collect();
        for blob in blobs_to_copy {
            let blob_name = get_blob_from_digest(blob.digest()).unwrap();
            let blob_path = p.join("blobs/sha256").join(blob_name);
            let src = fs::canonicalize(blob_path).unwrap();
            let dst = self.blob_dir.join(blob_name);
            fs::copy(src, dst).unwrap();
        }

        index
    }

    pub fn set_image_tag(&self, image: &ImageManifest, tag: &str) {
        // write the image to the dir, get the descriptor and add
        // it to the index
        let (mut desc, blob) = get_descriptor(&DescriptorLike::Image(image)).unwrap();
        let mut annotations: HashMap<String, String> = HashMap::new();
        annotations.insert(
            String::from("org.opencontainers.image.ref.name"),
            String::from(tag),
        );
        desc.set_annotations(Some(annotations));
        self.write_descriptor(&desc, blob);

        if let Ok(mut index) = ImageIndex::from_file(self.base.join("index.json")) {
            let mut manifests = index.manifests().clone();
            manifests.clear();
            manifests.push(desc);
            index.set_manifests(manifests);

            index.to_file(self.base.join("index.json")).unwrap();
        }
    }

    pub fn add_image(&self, layers: Vec<Descriptor>) -> ImageManifest {
        let run_config = ConfigBuilder::default().build().unwrap();
        let config = ImageConfigurationBuilder::default()
            .config(run_config)
            .build()
            .unwrap();

        let (cds, blob) = get_descriptor(&DescriptorLike::Config(&config)).unwrap();
        self.write_descriptor(&cds, blob);

        let image = ImageManifestBuilder::default()
            .schema_version(SCHEMA_VERSION)
            .layers(layers)
            .config(cds)
            .build()
            .unwrap();

        let (descriptor, datablob) = get_descriptor(&DescriptorLike::Image(&image)).unwrap();
        self.write_descriptor(&descriptor, datablob);
        if let Ok(mut index) = ImageIndex::from_file(self.base.join("index.json")) {
            let mut manifests = index.manifests().clone();
            manifests.push(descriptor);
            index.set_manifests(manifests);

            index.to_file(self.base.join("index.json")).unwrap();
        }
        image
    }

    pub fn add_image_index(&self, images: Vec<ImageManifest>) -> ImageIndex {
        let image_descriptors: Vec<Descriptor> = images
            .iter()
            .map(|im| get_descriptor(&DescriptorLike::Image(im)).unwrap().0)
            .collect();
        let image_index = ImageIndexBuilder::default()
            .schema_version(SCHEMA_VERSION)
            .manifests(image_descriptors)
            .build()
            .unwrap();

        let (image_index_descriptor, image_index_blob) =
            get_descriptor(&DescriptorLike::ImageIndex(&image_index)).unwrap();

        self.write_descriptor(&image_index_descriptor, image_index_blob);

        image_index
    }
}

pub fn make_oci_dir(name: &str) -> Result<OciDir> {
    fs::create_dir_all(name)?;
    let base = PathBuf::from(name);
    let blob_dir: PathBuf = [name, "blobs", "sha256"].iter().collect();
    fs::create_dir_all(&blob_dir)?;
    let oci_layout_file: PathBuf = [name, "oci-layout"].iter().collect();
    let mut file = fs::File::create(oci_layout_file)?;
    file.write_all(b"{\"imageLayoutVersion\":\"1.0.0\"}")?;
    Ok(OciDir { base, blob_dir })
}

pub fn make_layers_from_tars(tars: Vec<PathBuf>) -> Result<Vec<(Descriptor, PathBuf)>> {
    Ok(tars
        .iter()
        .map(|t| {
            let file = fs::File::open(t).expect("File should really be there");
            let (digest, size) = sha256_digest(file).expect("calculating hash");
            (
                DescriptorBuilder::default()
                    .media_type(MediaType::ImageLayer)
                    .size(i64::try_from(size).expect("Getting size"))
                    .digest(format!("sha256:{}", digest))
                    .build()
                    .expect("Adding Layer"),
                PathBuf::from(t),
            )
        })
        .collect())
}
