#!/bin/bash

set -xe
name=$1
dir=$PWD/tests

# Take original technique an make a json
$dir/ncf ncf-to-json $dir/techniques/${name}.cf

# Take json and produce a rudder-lang technique
cargo run -- --translate -i $dir/translations/${name}.json -o $dir/translations/${name}.rl

# Take rudder lang technique and compile it into cf file
cargo run -- --technique -i $dir/techniques/${name}.rl -o $dir/techniques/${name}.rl

# take generated cf file a new json
$dir/ncf ncf-to-json $dir/techniques/${name}.rl.cf

# TODO compare generated json
$dir/ncf compare-json $dir/translations/${name}.json $dir/translations/${name}.rl.json

# TODO compare generated cf files
$dir/ncf compare-cf $dir/techniques/${name}.cf $dir/techniques/${name}.rl.cf
