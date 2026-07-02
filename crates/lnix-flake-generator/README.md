# lnix-flake-generator

Parse `lazynix.yaml` and render `flake.nix`.

## Overview

This crate performs the core YAML-to-Nix transformation.

It reads the config into the `lnix-domain` AST.

It renders that AST into a `flake.nix` string.

## Background

The generated flake has a fixed shape.

It has stable, unstable, and pinned package inputs.

It has a `shellHook` assembled from several fragments.

Building that string in one large function was hard to follow.

So the rendering is split by concern.

## Purpose

Turn a validated `Config` into a correct `flake.nix`.

The output is deterministic.

The same config always renders the same flake.

## How it works

The public surface is small.

`LazyNixParser` reads and writes `lazynix.yaml`.

`render_flake` turns a `Config` into a `String`.

Rendering is split into focused modules under `generator/`:

- `path` resolves `~/`, absolute, and relative paths at shell-hook runtime
- `shell_hook` composes the ordered hook fragments
- `test_runner` renders the `LAZYNIX_TEST_MODE` test block
- `pinned` renders the per-package inputs and bindings
- `build_inputs` groups the `buildInputs` list
- `mod` orchestrates the final template

There is no intermediate representation.

The AST maps directly to the output string.

## Example

```rust,ignore
use lnix_flake_generator::{LazyNixParser, render_flake};
use std::path::PathBuf;

// Read lazynix.yaml from a directory.
let parser = LazyNixParser::new(PathBuf::from("."));
let config = parser.read_config()?;

// Render flake.nix. The override pins a different stable channel.
let flake: String = render_flake(&config, Some("github:NixOS/nixpkgs/nixos-25.06"));
```
