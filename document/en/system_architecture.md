# System Architecture

LazyNix is structured as a Rust workspace with four crates. Each crate has a single, well-defined responsibility, and dependencies flow in one direction: from the CLI entry point down to the lower-level libraries. This chapter explains what each crate does, how they connect, and how data flows through the system when you run a command.

## Architecture Overview

The workspace is organized under `src/`:

```
src/
  cli/              # Entry point and command routing
  flake-generator/  # YAML parsing and flake.nix generation
  linter/           # Package validation via nix eval
  nix-dispatcher/   # Nix command execution
  templates/        # Template files for lnix init
```

The dependency graph flows strictly top-down:

```
              cli
           /   |   \
          v    v    v
  flake-     linter   nix-
  generator             dispatcher
```

`cli` depends on all three library crates. The library crates do not depend on each other. This flat dependency structure keeps modules loosely coupled: changing how Nix commands are executed (in `nix-dispatcher`) does not affect how YAML is parsed (in `flake-generator`), and vice versa.

## cli

**Crate:** `lnix` (binary)
**Responsibility:** Parse CLI arguments, orchestrate subcommands, coordinate the other crates.

The `cli` crate is the only binary in the workspace. It defines the `lnix` command and its subcommands using [clap](https://docs.rs/clap/):

| Subcommand | Description |
|------------|-------------|
| `init`     | Create `lazynix.yaml` and `flake.nix` from templates |
| `develop`  | Generate `flake.nix` and enter a Nix development shell |
| `run`      | Execute a command inside the development environment |
| `test`     | Run test commands defined in `lazynix.yaml` |
| `task`     | Run a named task defined in `lazynix.yaml` |
| `update`   | Update `flake.lock` |
| `lint`     | Validate packages using `nix eval` |

Each subcommand follows the same pattern:

1. Read configuration (via `flake-generator`)
2. Validate the configuration
3. Generate `flake.nix` if needed (via `flake-generator`)
4. Execute a Nix command (via `nix-dispatcher`)

The `cli` crate does not contain business logic. It is a thin coordination layer that delegates work to the library crates. Error handling follows the railway pattern: each step returns a `Result`, and errors propagate upward to `main()` where they are printed and converted to an exit code.

### Key modules

- `cli_parser.rs` --- Defines the `Cli` struct and `Commands` enum with clap derive macros.
- `commands/` --- Subcommand implementations that are complex enough to warrant their own module (e.g., `lint`).
- `env_validator.rs` --- Validates environment variable configuration (dotenv file existence, etc.).
- `task_interpolator.rs` --- Substitutes CLI arguments into task command templates.

## flake-generator

**Crate:** `lnix-flake-generator` (library)
**Responsibility:** Parse `lazynix.yaml` and generate `flake.nix` content.

This crate handles the core translation from YAML to Nix. It exposes three public functions:

```rust
// Parse lazynix.yaml into a Config struct
let parser = LazyNixParser::new(config_dir);
let config: Config = parser.read_config()?;

// Validate the config (check for empty packages, invalid names, etc.)
validate_config(&config)?;

// Render the Config into a flake.nix string
let flake_content: String = render_flake(&config, override_url);
```

### Data model

The `Config` struct mirrors the `lazynix.yaml` structure:

```
Config
  └── DevShell
        ├── allow_unfree: bool
        ├── Package { stable, unstable }
        ├── shell_hook: Vec<String>
        ├── env: Env { dotenv, envvar }
        ├── test: Vec<String>
        └── task: HashMap<String, TaskDef>
```

The parser deserializes YAML into this struct using [serde](https://serde.rs/). The generator then walks the struct to produce a `flake.nix` string. There is no intermediate representation --- the translation is direct from the data model to the output string.

### Validation

`validate_config` checks constraints that YAML syntax alone cannot enforce:

- At least one package must be declared
- Package names must not be empty strings
- Duplicate package names across stable and unstable are flagged

Validation runs before generation, so invalid configurations never produce a `flake.nix`.

## linter

**Crate:** `lnix-linter` (library)
**Responsibility:** Verify that declared packages exist in nixpkgs before the user enters a shell.

The linter uses `nix eval` to check whether each package in `lazynix.yaml` can be resolved. This catches typos and platform-incompatible packages early, before the user waits for a slow `nix develop` invocation to fail.

```
Input: package names + target architecture
  │
  ├── nix_eval::eval_package()     # Run nix eval for each package
  ├── error_classifier::classify() # Categorize nix eval failures
  ├── validator::validate()        # Aggregate results
  └── reporter::format()           # Format human-readable output
```

### Key design decisions

- **Parallel evaluation.** The linter uses [rayon](https://docs.rs/rayon/) to evaluate multiple packages concurrently. This matters when `lazynix.yaml` declares many packages.
- **Error classification.** Raw `nix eval` errors are parsed into categories (package not found, attribute path error, architecture incompatibility) so the user gets an actionable message rather than a wall of Nix evaluation output.
- **Architecture awareness.** By default, the linter checks packages for the current system architecture. The `--arch` flag allows checking for a different target (e.g., verifying that a configuration works on `x86_64-linux` from an `aarch64-darwin` machine).

## nix-dispatcher

**Crate:** `lnix-nix-dispatcher` (library)
**Responsibility:** Execute Nix commands as subprocesses.

This crate provides a clean interface between LazyNix and the Nix CLI. It abstracts the details of spawning Nix processes, handling their exit codes, and reporting errors.

The public API is a set of functions:

| Function | What it does |
|----------|-------------|
| `run_nix_develop()` | Enter an interactive `nix develop` shell |
| `run_nix_develop_command(args)` | Run a single command inside `nix develop` |
| `run_flake_update()` | Execute `nix flake update` |
| `run_nix_test()` | Run test commands with `LAZYNIX_TEST_MODE=1` |
| `run_task_in_nix_env(commands)` | Execute multiple commands sequentially |

Each function constructs the appropriate `nix` command, spawns it as a subprocess, and returns the exit code. The crate does not interpret the output of Nix commands --- it only cares about success or failure.

### Error handling

All functions return `Result<T, NixDispatcherError>`. The error type covers two cases:

- **Command not found.** The `nix` binary is not on `PATH`.
- **Execution failure.** The subprocess could not be spawned (permission denied, etc.).

Note that a non-zero exit code from `nix` is not treated as an error in the Rust sense. It is returned as a value (`i32`) so the caller can decide how to handle it --- typically by forwarding it as the `lnix` process exit code.

## Data Flow

To tie it all together, here is what happens when you run `lnix develop`:

```
User runs: lnix develop
  │
  1. cli parses arguments (clap)
  │
  2. cli reads lazynix-settings.yaml (optional)
  │   └── This optional file allows overriding the nixpkgs version
  │       used for stable packages (e.g., pointing to a custom fork).
  │       Most users do not need this file.
  │
  3. flake-generator reads lazynix.yaml
  │   └── Deserializes into Config struct
  │
  4. flake-generator validates Config
  │   └── Returns error if packages are empty or invalid
  │
  5. cli validates env configuration
  │   └── Checks that referenced .env files exist
  │
  6. flake-generator renders flake.nix
  │   └── Writes the generated string to ./flake.nix
  │
  7. nix-dispatcher runs nix flake update (if --update)
  │
  8. nix-dispatcher runs nix develop
  │   └── Replaces the current process with an interactive shell
  │
  User is now inside the development environment.
```

Each step either succeeds and passes control to the next, or returns an error that propagates up to `main()`. There are no retries, no fallbacks, and no hidden state. The flow is linear and predictable.

## Summary

| Crate | Type | Responsibility |
|-------|------|---------------|
| `cli` | binary | Argument parsing, command orchestration |
| `flake-generator` | library | YAML parsing, validation, Nix code generation |
| `linter` | library | Package existence verification via `nix eval` |
| `nix-dispatcher` | library | Nix command execution as subprocesses |

Dependencies flow in one direction. Library crates do not depend on each other. Each crate can be understood, tested, and modified independently.
