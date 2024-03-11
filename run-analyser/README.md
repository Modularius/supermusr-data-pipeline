# events-analyser

## Introduction

## Command Line Interface

```shell
trace-to-events [OPTIONS] --broker <BROKER> [COMMAND]
```

For instance:

```shell
trace-to-events --broker localhost:19092 --trace-topic Trace --event-topic Events --group trace-to-events
```

The trace topic is the kafka topic that trace messages are consumed from, and event topic is the topic that event messages are produced to.

For instructions run:

```shell
trace-to-events --help
```
