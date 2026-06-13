# lnix-linter

Validate that declared packages exist, before you enter the shell.

## Overview

This crate checks each package in `lazynix.yaml`.

It uses `nix eval` to confirm the package resolves.

It reports problems in a human-friendly form.

## Background

A typo in a package name is easy to make.

Without a check, you find out only when `nix develop` fails.

That failure is slow and the error is a wall of Nix output.

The linter catches these problems early.

It turns raw evaluation errors into actionable messages.

## Purpose

Fail fast, and fail clearly.

Tell the user which package is wrong and why.

Point them to where they can look it up.

## How it works

The flow is a short pipeline.

- `nix_eval` runs `nix eval nixpkgs#<package>` for each package
- `error_classifier` sorts a failure into a category
- `validator` aggregates results across packages
- `reporter` formats the outcome for the terminal

Two design decisions matter.

First, evaluation runs in parallel with [rayon](https://docs.rs/rayon/).

This matters when many packages are declared.

Second, errors are classified, not dumped.

A failure becomes "package not found" or "unsupported architecture".

The `--arch` flag checks a target other than the current system.

For example, validate an `x86_64-linux` config from an `aarch64-darwin` machine.

Package names arrive as `lnix_core::PackageName`.

So shell-injection safety is a type guarantee, not a runtime check.

## Example

```rust,ignore
use lnix_core::PackageName;
use lnix_linter::{validate_packages, format_validation_result};

let packages: Vec<PackageName> =
    vec!["vim".parse().unwrap(), "git".parse().unwrap()];

let result = validate_packages(&packages, None);
print!("{}", format_validation_result(&result));
```
