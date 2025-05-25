#!/bin/sh
# generate sqlx metadata on main and test codes

cargo sqlx prepare
cargo sqlx prepare -- --tests
