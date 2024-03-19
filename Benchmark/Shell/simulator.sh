. ./output/Saves/Tests/Shell/lib.sh

compare_simple_and_advanced() {
    tteMode+=("constant-phase-discriminator --threshold=50 --duration=1 --cool-off=0")
    tteMode+=("advanced-muon-detector --muon-onset=0.1 --muon-fall=-0.1 --muon-termination=0.01 --duration=5  --smoothing-window-size=1")
}

simple_compare_threshold_test() {
    tteMode+=("constant-phase-discriminator --threshold=50 --duration=1 --cool-off=0")
    tteMode+=("constant-phase-discriminator --threshold=20 --duration=1 --cool-off=0")
    tteMode+=("constant-phase-discriminator --threshold=4  --duration=1 --cool-off=0")
}

advanced_compare_duration_and_smoothing_test() {
    tteMode+=("advanced-muon-detector --muon-onset=0.1 --muon-fall=-0.1 --muon-termination=0.01 --duration=5  --smoothing-window-size=1" )
    tteMode+=("advanced-muon-detector --muon-onset=0.1 --muon-fall=-0.1 --muon-termination=0.01 --duration=5  --smoothing-window-size=2" )
    tteMode+=("advanced-muon-detector --muon-onset=0.1 --muon-fall=-0.1 --muon-termination=0.01 --duration=10 --smoothing-window-size=10")
}

NUM_REPEATS=5
#simulator
#trace-reader

tteMode=()
simple_compare_threshold_test
run_test 0 $NUM_REPEATS simulator
#run_test 0 $NUM_REPEATS trace-reader

tteMode=()
advanced_compare_duration_and_smoothing_test
run_test 1 $NUM_REPEATS simulator
#run_test 1 $NUM_REPEATS trace-reader

tteMode=()
compare_simple_and_advanced
#run_test 2 $NUM_REPEATS simulator
#run_test 2 $NUM_REPEATS trace-reader