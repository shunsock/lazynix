# lnix-infra

The infrastructure layer of LazyNix: concrete adapters for the domain ports.

## Overview

This crate implements every trait in `lnix_domain::interface`.

Filesystem adapters live under `persistence/`.

Subprocess adapters for `nix` live under `gateway/`.

The terminal sink lives under `output/`.

It depends only on `lnix-domain` (plus serde for file formats).

## Background

I/O used to be scattered: `std::fs` calls in three crates, `Command::new("nix")` in two, `println!` everywhere.

Paths were read from the current working directory, so `flake.nix` landed in the CWD even when `--config-dir` pointed elsewhere.

Tests had to rewrite the CWD and serialize with `serial_test`.

## Purpose

One place per kind of I/O, behind a port.

Use-cases stay pure orchestration; this crate owns how files are read, how `nix` is spawned, and how messages reach the terminal.

## How it works

`WorkspacePaths` anchors every file to the config directory.

The composition root builds it once and injects it into each adapter, so nothing reads the CWD and tests run in parallel against temp dirs.

Subprocess execution funnels through two private helpers.

`run_inherit` hands the terminal to the child and returns the exit code.

`run_capture` collects stdout/stderr for the caller to interpret.

The `lnix init` templates are embedded here (`templates/`) via `include_str!` in the scaffolder adapter.

## Example

Adapters are constructed once and lent to use-cases through `lnix_app::Deps`.

```rust,ignore
use lnix_infra::persistence::{FsConfigRepository, FsFlakeWriter};
use lnix_infra::WorkspacePaths;

let paths = WorkspacePaths::new("./configs");
let repo = FsConfigRepository::new(paths.clone());
let writer = FsFlakeWriter::new(paths);

let config = repo.read_config()?;
```
