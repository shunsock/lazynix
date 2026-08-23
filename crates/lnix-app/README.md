# lnix-app

The application layer of LazyNix: use-cases, their dependency bundle, and the application error.

## Overview

This crate sits between the domain and everything else.

It depends only on `lnix-domain`.

It defines four things today.

First, `Deps`: the borrowed bundle of ports every use-case receives.

Second, `ApplicationError`: the top of the two-tier error hierarchy.

Third, `UseCaseEvent`: the semantic event vocabulary use-cases emit while they work — a config was read, a lock was updated, a task started — instead of assembling user-facing text themselves.

Fourth, `ReporterPort`: the port a use-case emits events through. Presentation adapters (terminal, JSON, TUI, i18n) map events to output.

Use-cases live under `usecase`, one per subcommand.

## Background

Command logic used to call `std::fs`, spawn `nix`, and print to stdout directly.

Testing a command meant a real filesystem, a real `nix` binary, and serialized tests.

The domain layer now defines ports for all of that I/O.

This crate is where the ports get consumed.

## Purpose

Make use-cases pure orchestration.

A use-case receives `Deps`, calls ports and pure domain services, and returns `Result<i32, ApplicationError>`.

Swapping mocks in tests is a plain struct literal.

## How it works

`Deps` holds one `&dyn` reference per port.

The composition root (the `lnix` binary) constructs concrete adapters once and lends them to each command.

`ApplicationError` lifts the domain's focused errors (`ConfigError`, `FlakeError`, `NixError`) via `#[from]`.

So `?` inside a use-case is the railway: any failure short-circuits with its category intact.

## Example

A use-case body stays free of I/O details and never assembles user-facing text. Progress and results are emitted as `UseCaseEvent`s; the composition root's presenter decides how to render them.

```rust,ignore
use lnix_app::{ApplicationError, Deps, UseCaseEvent};
use lnix_domain::render_flake;

fn develop(d: &Deps) -> Result<i32, ApplicationError> {
    let config = d.repo.read_config()?;                       // ConfigError -> ApplicationError
    let flake = render_flake(&config, None);                  // pure domain service
    d.writer.write_flake(&flake)?;                            // FlakeError -> ApplicationError
    d.reporter.report(&UseCaseEvent::EnteringDevelopShell);   // semantic event, no strings here
    d.nix.develop()?;                                         // NixError -> ApplicationError
    Ok(0)
}
```
