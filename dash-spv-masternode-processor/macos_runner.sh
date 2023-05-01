#!/bin/bash
echo "MacOS runner: start (masternodes-diff-processor) $1"
pwd
rm -fv target/debug/deps/dash-spv-*.keychain*
identity=$(security find-identity -p codesigning -v | grep -oE "Apple Development: (.*?) \(M62AAKG43G\)" -m 1)
echo "MacOS runner: identity: $identity"
codesign -s "$identity" --entitlements dash-spv.entitlements -fv "$1"
echo "MacOS runner: signed"
codesign --verify --verbose=4 "$1"
echo "MacOS runner: verified"
exec "$@"
