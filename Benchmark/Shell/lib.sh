EVENT_ANALYSER="cargo run --release --bin events-analyser --"
TRACE_TO_EVENTS="cargo run --release --bin trace-to-events --"
SIMULATOR="cargo run --release --bin simulator --"

#EVENT_ANALYSER="target/release/events-analyser"
#TRACE_TO_EVENTS="target/release/trace-to-events"
#SIMULATOR="target/release/simulator"
TRACE_READER="target/release/trace-reader"

            #--save-file output/Saves/Tests/Temp/Experiment$EXPR/Test$i/output_ \

MY_RUST_LOG=info
#LOGGER=""

create_log() {
    rm -f $1
    touch $1
}

run_test() {
    EXPR=$1
    NUM_REPEATS=$2
    INPUT=$3

    [ $INPUT = simulator ] \
    && TRACE_TO_EVENTS_INPUT_MODE="--polarity positive --baseline=0" \
    || TRACE_TO_EVENTS_INPUT_MODE="--polarity negative --baseline=0"
    [ $INPUT = simulator ] \
    && EVENT_ANALYSER_SIMULATED_TOPIC="--simulated-events-topic SimulatedEvents" \
    || EVENT_ANALYSER_SIMULATED_TOPIC=""

    echo "--" "Experiment" $EXPR
    echo "--" "Running" ${#tteMode[@]} "Event Formation Tests and " $TRACE_TO_EVENTS_INPUT_MODE " with" $NUM_REPEATS "repetition(s)"

    pkill trace-to-events
    pkill events-analyser

    for ((i=1; i <= ${#tteMode[@]}; i++)); do
        echo "-- --" "Event Formation Test" $i "with" ${tteMode[$(($i - 1))]}

        create_log output/Saves/Tests/Temp/Logs/ev_an$EXPR.$i.log

        RUST_LOG=$MY_RUST_LOG $EVENT_ANALYSER \
            --broker localhost:19092 --group events-analyser$i \
            --trace-to-events-topic Events$i $EVENT_ANALYSER_SIMULATED_TOPIC \
            --expected-repetitions=$NUM_REPEATS \
            --path output/Saves/Tests/Data/data.$EXPR.$i.csv &

        create_log output/Saves/Tests/Temp/Logs/tr_ev$EXPR.$i.log

        RUST_LOG=$MY_RUST_LOG $TRACE_TO_EVENTS \
            --broker localhost:19092 --group trace-to-events$i \
            --trace-topic Traces --event-topic Events$i \
            --observability-address=127.0.0.1:909$i \
            $TRACE_TO_EVENTS_INPUT_MODE \
            ${tteMode[$(($i - 1))]}  &
    done

    create_log output/Saves/Tests/Temp/Logs/trace$EXPR.log

    echo "--" "Running" $INPUT
    [ $INPUT = simulator ] \
    && RUST_LOG=$MY_RUST_LOG $SIMULATOR \
            --broker localhost:19092 \
            --trace-topic Traces \
            --event-topic SimulatedEvents \
            json \
            --path "Benchmark/data.json" \
            --repeat=$NUM_REPEATS  \
    || RUST_LOG=$MY_RUST_LOG $TRACE_READER \
            --broker localhost:19092 \
            --trace-topic Traces \
            --frame-number=0 \
            --digitizer-id=0 \
            --number-of-trace-events=5 \
            --frame-interval-ms=20 \
            --repeat=$NUM_REPEATS \
            --path "../Data/Traces/MuSR_A41_B42_C43_D44_Apr2021_Ag_ZF_IntDeg_Slit60_short.traces"

    echo "--" $INPUT "finished"

    sleep 10

    echo "--" "Ending Experiment" $EXPR

    pkill trace-to-events
    pkill events-analyser
}