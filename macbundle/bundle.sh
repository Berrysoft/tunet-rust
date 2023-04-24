#!/usr/bin/env bash

current=$(dirname "${BASH_SOURCE[0]}")
workspace=${current}/..

x86_target=${workspace}/target/x86_64-apple-darwin/release
aarch64_target=${workspace}/target/aarch64-apple-darwin/release

bundle_target=${workspace}/target/macbundle
mkdir -p ${bundle_target}
echo Creating bundle at ${bundle_target}

app_dir=${bundle_target}/tunet.app
content_dir=${app_dir}/Contents

echo Creating Info.plist
mkdir -p ${content_dir}
cp ${current}/Info.plist ${content_dir}/

echo Creating logo.icns
resource_dir=${content_dir}/Resources
mkdir -p ${resource_dir}
cp ${workspace}/logo.icns ${resource_dir}/

binary_dir=${content_dir}/MacOS
mkdir -p ${binary_dir}

binaries=(tunet tunet-cui tunet-gui tunet-service)
for b in "${binaries[@]}"
do
    echo Creating $b
    lipo -create ${x86_target}/$b ${aarch64_target}/$b -output ${binary_dir}/$b
done

echo Zipping
cd ${bundle_target}
zip -r tunet.app.zip tunet.app
