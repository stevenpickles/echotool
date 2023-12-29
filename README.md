# echotool

This Rust based command line utility can run as both an Echo server and an Echo client and fulfills the functionality described in [RFC 862](https://www.rfc-editor.org/rfc/rfc862).

The source compiles with no `cargo build` warnings and no `cargo clippy` warnings or recommendations. Command line parameters are parsed and matched with `clap`. Instead of `println()` or `eprintln()` calls the project uses `env-logger`. The logger is configured out of the box to show all info and higher level messages to the terminal.

Compilation and runtime functionality is tested on Linux, Mac, and Windows.

## Command Line Options

The `-h/--help` option provides a description, defaults, and options for each command line parameter:

<pre>
$ ./echotool -h
<b><u>Usage:</u> echotool</b> [OPTIONS] [remote_url]

<b><u>Arguments:</b></u>
  [remote_url]  the remote IP address or URL to connect to (client mode only); omit for server modes

Options:
  <b>-r, --remote_port</b> &lt;remote_port&gt;     the remote port to connect to (client modes only) [default: 7]
  <b>-l, --local_port</b> &lt;local_port&gt;       the local port to bind to (client and server modes) [default: 7]
  <b>-d, --data_payload</b> &lt;data_payload&gt;   the data payload to send (client modes only) [default: "Hello World!"]
  <b>-c, --count</b> &lt;count&gt;                 the number of times to send the data payload (client modes only) [default: 5]
  <b>-t, --timeout</b> &lt;timeout_in_seconds&gt;  the timeout in seconds (client modes only) [default: 1.0]
  <b>-p, --protocol</b> &lt;protocol&gt;           the protocol to use (client and server modes) [default: udp] [possible values: udp, tcp]
  <b>-h, --help</b>                          Print help
  <b>-V, --version</b>                       Print version
</pre>

## `clippy` Options

```bash
$ cargo clippy -- -W clippy::pedantic -W clippy::nursery -W clippy::unwrap_used -W clippy::expect_used
```

