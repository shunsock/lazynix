# lnix-nix-dispatcher

Nix command dispatcher for LazyNix.

## Overview

This crate provides a clean interface for executing Nix commands within the LazyNix environment. It handles command execution, error handling, and provides a consistent API for Nix operations.

## Features

- Execute `nix develop` shells
- Run commands within Nix development environments
- Update flake.lock files
- Run tests in Nix environments
- Execute tasks with command interpolation

## Usage

```rust
use lnix_nix_dispatcher::{run_nix_develop, run_nix_develop_command};

// Enter nix develop shell
run_nix_develop()?;

// Run a command in nix develop environment
let exit_code = run_nix_develop_command(vec!["cargo".to_string(), "build".to_string()])?;
```

## API

- `run_nix_develop()` - Enter an interactive nix develop shell
- `run_flake_update()` - Update flake.lock file
- `run_nix_develop_command(cmd_args)` - Execute a command in nix develop
- `run_nix_test()` - Run tests in nix environment with LAZYNIX_TEST_MODE=1
- `run_task_in_nix_env(commands)` - Execute multiple commands sequentially

## Error Handling

All functions return `Result<T, NixDispatcherError>` for consistent error handling.
