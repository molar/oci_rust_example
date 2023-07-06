#!/bin/bash


rm -rf temp_dir/
cargo run -q -- mutate tests/data/base_oci_dir --tag a_tag  --output temp_dir/oci_dir --append tests/data/a.tar --append tests/data/b.tar --platform x86_64
tree temp_dir/oci_dir
cat temp_dir/oci_dir/index.json
