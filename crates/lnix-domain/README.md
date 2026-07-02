# lnix-domain

The domain layer of LazyNix: value objects, the configuration AST, pure services, and ports.

## Overview

This crate is the innermost layer of the workspace.

It defines four things.

First, value objects such as `PackageName` and `RegistryUrl`.

Second, the `Config` AST that mirrors `lazynix.yaml`.

Third, pure services under `service/`: flake rendering, lint classification and reporting, task-command interpolation.

Fourth, ports under `interface/`: the traits that infrastructure adapters implement (persistence, gateway, output), with the focused error types they return.

It performs no I/O.

It depends only on `serde` and `thiserror`.

## Background

Earlier, package-name validation was duplicated in three places.

The flake generator, the linter, and the CLI each had their own check.

The rules drifted apart over time.

A `String` cannot tell you whether it was ever validated.

So a missing check was easy to introduce and hard to notice.

## Purpose

Make illegal states unrepresentable.

A value that exists in the program is already valid.

Downstream code never re-checks it.

## How it works

Each value object wraps a `String`.

It validates its invariant in `TryFrom<String>`.

It is wired into `serde` with `#[serde(try_from = "String", into = "String")]`.

So a successful deserialization *is* the proof of validity.

An invalid name is rejected at YAML parse time, not later.

Cross-field constraints that a single type cannot express live in `validate_config`.

For example: a task must have at least one command.

## Example

A package name validates itself.

```rust,ignore
use lnix_domain::PackageName;

// Accepted: alphanumerics, '-', '_', and '.' for nested attributes.
let name: PackageName = "python312Packages.pip".parse().unwrap();

// Rejected: shell metacharacters cannot form a PackageName.
assert!("pkg; rm -rf /".parse::<PackageName>().is_err());
```

Parsing a config rejects an invalid task name before any flake is generated.

```rust,ignore
use lnix_domain::Config;

let yaml = r#"
devShell:
  package:
    stable:
      - name: bash
  task:
    "invalid@name":
      commands:
        - echo hi
"#;

assert!(serde_yaml::from_str::<Config>(yaml).is_err());
```
