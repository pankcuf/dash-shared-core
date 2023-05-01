#!/bin/bash
cargo build --lib
dir=target/debug/deps

last_modified_file=$(ls -lat target/debug/deps | grep -oE 'dash_spv_masternode_processor-[a-f0-9]{16}$')
exec_path=$dir/"$last_modified_file"

echo "The last modified file in $dir is: $last_modified_file"
identity=$(security find-identity -p codesigning -v | grep -oE "Apple Development: (.*?) \(M62AAKG43G\)" -m 1)
codesign --display --verbose=4 "$exec_path"
#codesign --entitlements - -r - "$exec_path"
#codesign --force --sign "$identity" --options runtime --timestamp --entitlements dash-spv.entitlements "$exec_path";
codesign -f -r --sign "$identity" --options runtime --entitlements - "$exec_path";
codesign --verify "$exec_path"
cargo test --lib
