# rtt-target

(Temporary name - suggest a better one!)

This crate implements the RTT (Real-Time Transfer) debugging protocol. It is platform agnostic and
should be usable on any microcontroller, however whether it is actually useful depends on if the
chip implements a background memory interface and if there is an RTT host program for it.

## Running the examples

To run the examples you will have to provide any needed platform specific configuration such as
`.cargo/config` or `memory.x` yourself. Additionally you will to specify a feature to enable the
example dependencies, such as `--feature examples-cortex-m`.

## TODO

- Virtual terminal support for channel 0
- Non-blocking writes

## License

This software is licensed under the MIT license.

The SEGGER RTT protocol is used for compatibility with existing software, however this project is
not affiliated with nor uses any code belonging to SEGGER Microcontroller GmbH.
