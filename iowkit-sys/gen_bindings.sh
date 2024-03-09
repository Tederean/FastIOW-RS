#!/bin/bash

bindgen --ctypes-prefix raw --no-doc-comments --no-layout-tests --raw-line '#![allow(non_upper_case_globals)]' \
    --raw-line '#![allow(non_camel_case_types)]' --raw-line '#![allow(non_snake_case)]' \
    --raw-line '#![allow(dead_code)]'  --raw-line 'use std::os::raw;' --rustfmt-bindings \
    --dynamic-loading Iowkit -o genned_bindings.rs iowkit.h

# final format after using sed on the bindings
rustfmt genned_bindings.rs
