#!/bin/bash

# (c) Copyright 2021 Christian Saide
# SPDX-License-Identifier: GPL-3.0

BUILD="${1}"
TARGET="${2}"
STRIP="${3}"

for bin in $(find target/${TARGET}/${BUILD}/* -maxdepth 0 -type f -executable); do
    ${STRIP} -s "${bin}"
    upx -9 -q "${bin}"
    upx -t "${bin}"
done