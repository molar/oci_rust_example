use liboci::{self, get_descriptor, DescriptorLike};
use oci_spec::image::{ImageIndex, ImageManifest};
use std::{fs::remove_dir_all, path::PathBuf};
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
        "sha256:1b28b01c5b6ff05b08a6e178fbd2af8f1f3d81af6e0ccd6f1e482f4eefc60156"
    );
    assert_eq!(
        info[1].0.digest(),
        "sha256:c2f91b7f766221c74813f76609ffd7b3c0b83719c4c957c09bd84901f819e1c7"
    );
}

#[test]
fn it_makes_oci_dir() {
    let tempdir = TempDir::new("test_image").expect("Failed to create temp dir");
    let oci_dir = liboci::make_oci_dir(tempdir.path().join("oci_repo").to_str().unwrap()).unwrap();
    assert_eq!(oci_dir.base, tempdir.path().join("oci_repo"))
}

#[test]
fn it_makes_oci_image() {
    let tempdir = TempDir::new("test_image").expect("Failed to create temp dir");
    let oci_dir = liboci::make_oci_dir(tempdir.path().join("oci_repo").to_str().unwrap()).unwrap();
    let tars = vec![
        PathBuf::from("tests/data/a.tar"),
        PathBuf::from("tests/data/b.tar"),
    ];
    let info = liboci::make_layers_from_tars(tars).expect("asdf");
    info.iter().for_each(|m| {
        oci_dir.link_descriptor(&m.0, &m.1).unwrap();
    });

    assert!(oci_dir
        .blob_dir
        .join("1b28b01c5b6ff05b08a6e178fbd2af8f1f3d81af6e0ccd6f1e482f4eefc60156")
        .exists());
    assert!(oci_dir
        .blob_dir
        .join("c2f91b7f766221c74813f76609ffd7b3c0b83719c4c957c09bd84901f819e1c7")
        .exists());

    let image = oci_dir.add_image(info.iter().map(|m| m.0.clone()).collect());
    assert_eq!(
        image.config().digest(),
        "sha256:bc894e1d83f844c4d0d01a17a04850d1b0f7e75cc3ead16660f3c15be58f6623"
    );

    let image_index = oci_dir.add_image_index(vec![image]);
    let (image_index_descriptor, _blob) =
        get_descriptor(&DescriptorLike::ImageIndex(&image_index)).unwrap();

    let image_index_read_path = oci_dir
        .get_descriptor_file(&image_index_descriptor)
        .unwrap();
    let image_index_read = ImageIndex::from_file(image_index_read_path).unwrap();
    assert_eq!(
        image_index_read.manifests().first().unwrap().digest(),
        image_index.manifests().first().unwrap().digest()
    );
}

#[test]
fn it_makes_oci_image_from_base() {
    let tempdir = TempDir::new("test_image").expect("Failed to create temp dir");
    let oci_dir = liboci::make_oci_dir(tempdir.path().join("oci_repo").to_str().unwrap()).unwrap();

    let base_dir = PathBuf::from("tests/data/base_oci_dir");
    let index = oci_dir.add_base_oci_dir(&base_dir);

    assert!(oci_dir
        .blob_dir
        .join("59d2663aa737ac5d6c007a9a4d828f77c721c4e25788da6a2aebd4c6299e8482")
        .exists());

    let busybox_img = index
        .manifests()
        .first()
        .unwrap()
        .annotations()
        .as_ref()
        .unwrap();
    assert!(busybox_img.contains_key("org.opencontainers.image.ref.name"));
    assert_eq!(
        busybox_img
            .get("org.opencontainers.image.ref.name")
            .unwrap(),
        "docker.io/library/busybox:latest"
    );
}

/// .
///
/// # Panics
///
/// Panics if .
#[test]
fn it_works() {
    remove_dir_all("temp/oci_dir").ok();
    let oci_dir = liboci::make_oci_dir("temp/oci_dir").unwrap();

    let base_dir = PathBuf::from("tests/data/base_oci_dir");
    let index = oci_dir.add_base_oci_dir(&base_dir);

    assert!(oci_dir
        .blob_dir
        .join("59d2663aa737ac5d6c007a9a4d828f77c721c4e25788da6a2aebd4c6299e8482")
        .exists());

    let mut busybox_img = ImageManifest::from_file(
        oci_dir
            .get_descriptor_file(index.manifests().first().unwrap())
            .unwrap(),
    )
    .unwrap();

    let tars = vec![
        PathBuf::from("tests/data/a.tar"),
        PathBuf::from("tests/data/b.tar"),
    ];
    let info = liboci::make_layers_from_tars(tars).expect("asdf");
    info.iter().for_each(|m| {
        oci_dir.link_descriptor(&m.0, &m.1).unwrap();
        busybox_img.layers_mut().push(m.0.clone());
    });

    //read and mutate the busybox image
    oci_dir.set_image(&busybox_img);
}
