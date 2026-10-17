#!/usr/bin/env bash
# Downloads the libvips build sharp 0.34.5 ships (sharp-libvips 1.2.4: libvips 8.17.3 with mozjpeg, libspng, libwebp,
# libheif with AV1 only, librsvg and cgif in one library) into rust/libvips/<platform>/, where the server crate links
# it, so images decode and encode exactly as in the JS server on every machine.
#
#   rust/fetch-libvips.sh [linux-x64|linux-arm64|darwin-arm64|darwin-x64]
set -euo pipefail

version=1.2.4
case "$(uname -s)-$(uname -m)" in
  Linux-x86_64) default=linux-x64 ;;
  Linux-aarch64) default=linux-arm64 ;;
  Darwin-arm64) default=darwin-arm64 ;;
  Darwin-x86_64) default=darwin-x64 ;;
  *) default= ;;
esac
platform=${1:-$default}
# sha512 of each npm tarball, as package-lock.json records it.
case "$platform" in
  linux-x64) sha512=b49c6288bb261dcf40c756f3de868e6015114d718844338306a86f7951e89eb1c9f7ffa4f3da9b2e5d1b7099ecf9ee2de2bbda341c5a119b05b527c076ab8faf ;;
  linux-arm64) sha512=7b17235fc0dfb08709d74c752b3af845c59ed5e742f4faae0d144fc776150af42ff94e69ed88a7dacdf67edce2917a236f53c815cffd32ddbcff2fa2464964af ;;
  darwin-arm64) sha512=cea8e3a3b45ab457c5a0fd0c910e758dfb856419d5136a5189ac9d289d46feb1d9be7b2b1c039c4002c88bdb00e5ca39c5e9d0753ba00afb5bd5cb9fefc55fe2 ;;
  darwin-x64) sha512=d4839de717d58651b05fecd7bf637dde4d3230e36f52500dca56c9c357936a1f0afc9b698b5e4a0be59289a5ff9c199b9b61f144cd606749d27634acad96c62a ;;
  *) echo "fetch-libvips: no sharp-libvips build for platform '${platform}'" >&2; exit 2 ;;
esac

out="$(cd "$(dirname "$0")" && pwd)/libvips/$platform"
work=$(mktemp -d)
trap 'rm -r "$work"' EXIT
curl -fsSL -o "$work/package.tgz" "https://registry.npmjs.org/@img/sharp-libvips-$platform/-/sharp-libvips-$platform-$version.tgz"
if command -v sha512sum >/dev/null; then got=$(sha512sum "$work/package.tgz"); else got=$(shasum -a 512 "$work/package.tgz"); fi
if [ "${got%% *}" != "$sha512" ]; then
  echo "fetch-libvips: the sharp-libvips-$platform $version tarball does not match its sha512" >&2
  exit 1
fi
tar -xzf "$work/package.tgz" -C "$work"
mkdir -p "$out"
cp "$work"/package/lib/libvips-cpp.* "$out/"
cp "$work/package/versions.json" "$out/"
echo "libvips $(sed -n 's/.*"vips": *"\([^"]*\)".*/\1/p' "$out/versions.json") in $out"
