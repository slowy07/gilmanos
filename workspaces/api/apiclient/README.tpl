# {{crate}}

Current version: {{version}}

## apiclient binary

The `apiclient` binary helps you talk to an HTTP API over a Unix-domain socket.

It talks to the Thar socket by default.
It can be pointed to another socket using `--socket-path`, for example for local testing.

The URI path is specified with `-u` or `--uri`, for example `-u /settings`.
This should include the query string, if any.

The HTTP method defaults to GET, and can be changed with `-m`, `-X`, or `--method`.

If you change the method to POST or PATCH, you may also want to send data in the request body.
Specify the data after `-d` or `--data`.

To see verbose response data, including the HTTP status code, use `-v` or `--verbose`.

### Example usage

Getting settings:

```
apiclient -m GET -u /settings
apiclient -m GET -u /settings/pending
```

Changing settings:

```
apiclient -X PATCH -u /settings -d '{"timezone": "OldLosAngeles"}'
apiclient -m POST -u /settings/commit_and_apply
```

## apiclient library

{{readme}}

## Colophon

This text was generated from `README.tpl` using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/lib.rs`.
