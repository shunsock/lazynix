# lnix

The LazyNix command-line tool.

This is the only binary crate in the workspace.

## Overview

`lnix` is a YAML-to-Nix transpiler for lazy engineers.

You describe a dev environment in `lazynix.yaml`.

`lnix` generates a `flake.nix` and drives `nix` for you.

## Background

A Nix `flake.nix` is powerful but verbose.

Writing one by hand is a barrier for many developers.

LazyNix trades a small, declarative YAML file for that boilerplate.

The generated `flake.nix` is disposable.

You edit the YAML, not the flake.

## Purpose

Be a thin orchestration layer.

This crate holds no business logic of its own.

It parses arguments and delegates to the library crates.

## How it works

`main.rs` parses arguments with [clap](https://docs.rs/clap/) and dispatches.

Each subcommand lives in its own module under `commands/`.

The commands that generate a flake share one pipeline.

`commands/pipeline.rs` holds that shared prefix:

1. read and validate the config (`lnix-domain`, `lnix-flake-generator`)
2. resolve pinned package versions (`lnix-nix-dispatcher`)
3. render and write `flake.nix` (`lnix-flake-generator`)

A command then adds its own tail.

`develop` enters the shell, `test` runs tests, `run` runs a command.

Errors follow a railway pattern.

Each step returns a `Result`, and errors bubble up to `main`.

## Subcommands

| Command | What it does |
|---------|--------------|
| `init` | Scaffold `lazynix.yaml` and `flake.nix` from templates. |
| `develop` | Generate `flake.nix` and enter `nix develop`. |
| `run` | Run a command inside the dev environment. |
| `test` | Run the test commands declared in `lazynix.yaml`. |
| `task` | Run a named task from `lazynix.yaml`. |
| `update` | Update `flake.lock`. |
| `lint` | Check that declared packages exist via `nix eval`. |
| `search` | Find available package versions via nix-versions. |

## Example

```bash
# Scaffold a project.
lnix init

# Edit lazynix.yaml, then enter the dev shell.
lnix develop

# Run a one-off command in the environment.
lnix run -- cargo build

# Check that every declared package resolves.
lnix lint
```
