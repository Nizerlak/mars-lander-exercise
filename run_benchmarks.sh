#!/bin/bash

# Helper script to run critetion benchmarks

while [[ $# -gt 0 ]]; do
    case "$1" in
        --solver-settings)
            export SOLVER_SETTINGS="${PWD}/$2"
            shift 2
            ;;
        --simulation-file)
            export SIMULATION_FILE="${PWD}/$2"
            shift 2
            ;;
        --max-iterations)
            export MAX_ITERATIONS="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Run the benchmark with the environment variables set
cargo bench -- solving_time
