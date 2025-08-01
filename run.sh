#!/bin/bash

binary_path="fuzzing_targets/command-line_tools/ls"
fuzz_type="txt"
max_iterations="1000"
timeout="100"

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

echo "RUST_LOG=$log_type cargo run -- -b $binary_path --fuzz-type $fuzz_type --max-iterations $max_iterations --timeout $timeout"

RUST_LOG=$log_type cargo run -- -b $binary_path --fuzz-type $fuzz_type --max-iterations $max_iterations --timeout $timeout
