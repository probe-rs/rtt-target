# rtt-target

Target side implementation of the RTT (Real-Time Transfer) I/O protocol RTT implements input and output to/from a debug probe using in-memory ring buffers and memory polling. This enables debug logging from the microcontroller with minimal delays and no blocking, making it usable even in real-time applications where e.g. semihosting delays cannot be tolerated.

## Platform support

While this crate is platform agnostic, some platform specific code is needed if you want to use the global `rprintln!` macro.

If using Cortex-M, there is built-in support with a feature flag:

```toml
# Cargo.toml
rtt-target = { version = "x.y.z", features = ["cortex-m"] }
```

Otherwise, check the documentation for the `set_print_channel_cs` function.

## Running the examples

To run the examples you will have to provide any needed platform specific configuration such as `.cargo/config` or `memory.x` yourself. Additionally you must specify a feature to enable the example dependencies, such as `--feature examples-cortex-m`.

## License

This software is licensed under the MIT license.

The SEGGER RTT protocol is used for compatibility with existing software, however this project is not affiliated with nor uses any code belonging to SEGGER Microcontroller GmbH.
