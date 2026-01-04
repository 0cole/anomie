#!/bin/bash

# `which <executable>` to determine the executable's full path
binary_path="/usr/local/bin/magick"
fuzz_type="png"
# === Placeholders === 
# - {input} will be replaced when running with the actual mutated file's name,
# - {temp_dir} should be used to describe the temp_dir that the fuzzer uses
#   note that there is a scratch dir there to store output files, to write to it,
#   use `{temp_dir}/scratch`
args="{input} {temp_dir}/scratch/out.png"
max_iterations="1000"
timeout="5000"

if [ -z "$1" ]; then
  log_type="info"
elif [ "$1" == "debug" ]; then 
	log_type="debug"
elif [ "$1" == "info" ]; then 
	log_type="info"
elif [ "$1" == "warn" ]; then 
	log_type="warn"
elif [ "$1" == "trace" ]; then 
	log_type="trace"
elif [ "$1" == "error" ]; then 
	log_type="error"
fi

echo "RUST_LOG=$log_type cargo run -- -b $binary_path --fuzz-type $fuzz_type --max-iterations $max_iterations --timeout $timeout --\"$args\""

RUST_LOG=$log_type cargo run -- -b $binary_path --fuzz-type $fuzz_type --max-iterations $max_iterations --timeout $timeout -- "$args"
