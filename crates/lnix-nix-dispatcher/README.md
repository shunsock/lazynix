# lnix-nix-dispatcher

Run `nix` commands as subprocesses.

## Overview

This crate is the boundary between LazyNix and the `nix` CLI.

It launches `nix` processes.

It handles their exit codes and reports errors.

## Background

LazyNix drives Nix, it does not reimplement it.

Spawning subprocesses and threading exit codes is repetitive.

Centralizing it here keeps the rest of the codebase clean.

The CLI calls one function instead of building a `Command`.

## Purpose

Provide a small, consistent API for Nix operations.

Abstract away the details of process spawning.

Leave the interpretation of output to the caller.

## How it works

Each function builds the right `nix` invocation.

It launches it as a subprocess.

It returns either an exit code or an error.

This crate does not parse Nix output.

It only distinguishes success from failure.

A non-zero exit code from `nix` is not a Rust error.

It is returned as an `i32` so the caller can decide what to do.

Typically the caller forwards it as the `lnix` exit code.

## API

| Function | Behavior |
|----------|----------|
| `run_nix_develop()` | Enter an interactive `nix develop` shell. |
| `run_nix_develop_command(args)` | Run one command inside `nix develop`. |
| `run_flake_update()` | Run `nix flake update`. |
| `run_nix_test()` | Run tests with `LAZYNIX_TEST_MODE=1`. |
| `run_task_in_nix_env(commands)` | Run several commands sequentially. |
| `resolve_version(name, version)` | Resolve a pinned version via nix-versions. |
| `search_versions(...)` | Search available versions via nix-versions. |

## Example

```rust,ignore
use lnix_nix_dispatcher::{run_nix_develop, run_nix_develop_command};

// Enter an interactive dev shell.
run_nix_develop()?;

// Or run a single command inside it.
let exit_code = run_nix_develop_command(vec!["cargo".to_string(), "build".to_string()])?;
```

## Error handling

All functions return `Result<T, NixDispatcherError>`.

A missing `nix` binary and a failed spawn are the error cases.
