#!/bin/bash

set -xe
name=$1
dir=$PWD/tests

# Take original technique an make a json
# $dir/ncf ncf-to-json $dir/translate/${name}.cf

# Take json and produce a rudder-lang technique
cargo run -- --translate -i $dir/translate/${name}.json -o $dir/target/${name}.rl

# Take rudder lang technique and compile it into cf file
cargo run -- --technique -i $dir/compile/${name}.rl -o $dir/target/${name}.rl

# take generated cf file a new json
# $dir/ncf ncf-to-json $dir/compile/${name}.rl.cf

# TODO compare generated json
$dir/ncf compare-json $dir/translate/${name}.json $dir/target/${name}.rl.json

# TODO compare generated cf files
$dir/ncf compare-cf $dir/compile/${name}.cf $dir/target/${name}.rl.cf
