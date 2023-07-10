#!/bin/bash
set -euo pipefail

rm -rf temp_dir/
cargo run -q -- mutate tests/data/base_oci_dir --tag a_tag \
  --output temp_dir/oci_dir \
  --append tests/data/a.tar \
  --append tests/data/b.tar \
  --platform x86_64 \
  --env-file tests/data/env.txt \
  --annotations-file tests/data/annotations.txt \
  --labels-file tests/data/labels.txt \
  --user 1234 \
  --entrypoint /bin/ls,-l \
  --cmd /bin/true,-l \
  --workdir /root/a
tree temp_dir/oci_dir
cat temp_dir/oci_dir/index.json
