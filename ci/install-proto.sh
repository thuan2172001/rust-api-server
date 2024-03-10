#!/bin/sh

set -ex

die() {
    echo "$@" >&2
    exit 1
}

test -n "$PROTOBUF_VERSION" || die "PROTOBUF_VERSION env var is undefined"

case "$PROTOBUF_VERSION" in
3*)
    basename="protobuf-cpp-$PROTOBUF_VERSION"
    ;;
*)
    die "unknown protobuf version: $PROTOBUF_VERSION"
    ;;
esac

curl -sL "https://github.com/google/protobuf/releases/download/v$PROTOBUF_VERSION/$basename.tar.gz" | tar zx

cd "protobuf-$PROTOBUF_VERSION"

./configure --prefix="$HOME" && make -j2 && make install

test -n "$GITHUB_PATH"
echo "$HOME/bin" >>"$GITHUB_PATH"