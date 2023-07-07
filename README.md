# oci_rust_example
playing with oci and rust

## Goals
Use rust to create a oci dir containing multiple images
* eventually use this tool to create container images in bazel that are ready to run with podman.
* Use symlinks to avoid having to copy blobs into the blobs directory
* have the required cli flags for setting image properties such as env, args ,entrypoint, cmds, .... 
* mimics the api of crane used in rules_oci
