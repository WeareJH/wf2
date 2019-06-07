#!/usr/bin/env bash
docker run --init --rm -v ${PWD}:/build wearejh/rust-macos-build
