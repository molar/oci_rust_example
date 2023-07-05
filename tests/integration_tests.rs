use liboci::{self, get_descriptor, DescriptorLike};
use oci_spec::image::{ImageIndex, ImageManifest};
use std::path::PathBuf;
use tempdir::TempDir;

#[test]
fn it_makes_layers_from_tars() {
    let tars = vec![
        PathBuf::from("tests/data/a.tar"),
        PathBuf::from("tests/data/b.tar"),
    ];
    let info = liboci::make_layers_from_tars(tars).expect("asdf");
    assert_eq!(
        info[0].0.digest(),
        "sha256:46d550210427d38fce133124e45d8ee093df286c1b817cb168195d4f84303e92"
    );
    assert_eq!(
        info[1].0.digest(),
        "sha256:13df0db281ec8ae75fbf1532677f651062b4da5dc7d4586082067bf93aad0dc8"
    );
}

#[test]
fn it_makes_oci_dir() {
    let tempdir = TempDir::new("test_image").expect("Failed to create temp dir");
    let oci_dir = liboci::make_oci_dir(tempdir.path().to_str().unwrap()).unwrap();
    assert_eq!(oci_dir.base, tempdir.path())
}

#[test]
fn it_makes_oci_image() {
    let tempdir = TempDir::new("test_image").expect("Failed to create temp dir");
    let oci_dir = liboci::make_oci_dir(tempdir.path().to_str().unwrap()).unwrap();
    let tars = vec![
        PathBuf::from("tests/data/a.tar"),
        PathBuf::from("tests/data/b.tar"),
    ];
    let info = liboci::make_layers_from_tars(tars).expect("asdf");
    info.iter().for_each(|m| {
        oci_dir.link_descriptor(&m.0, &m.1);
    });

    assert!(oci_dir
        .blob_dir
        .join("46d550210427d38fce133124e45d8ee093df286c1b817cb168195d4f84303e92")
        .exists());
    assert!(oci_dir
        .blob_dir
        .join("13df0db281ec8ae75fbf1532677f651062b4da5dc7d4586082067bf93aad0dc8")
        .exists());

    let image = oci_dir.add_image(info.iter().map(|m| m.0.clone()).collect());
    assert_eq!(
        image.config().digest(),
        "sha256:bc894e1d83f844c4d0d01a17a04850d1b0f7e75cc3ead16660f3c15be58f6623"
    );

    let image_index = oci_dir.add_image_index(vec![image.clone()]);
    let (image_index_descriptor, blob) =
        get_descriptor(&DescriptorLike::ImageIndex { 0: &image_index });

    let image_index_read_path = oci_dir
        .get_descriptor_file(&image_index_descriptor)
        .unwrap();
    let image_index_read = ImageIndex::from_file(image_index_read_path).unwrap();
    assert_eq!(
        image_index_read.manifests().first().unwrap().digest(),
        image_index.manifests().first().unwrap().digest()
    );
}
