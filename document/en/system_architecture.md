# System Architecture

LazyNix is a Rust workspace made of four crates. Each crate owns a single responsibility, and dependencies flow strictly from the CLI down to a pure domain core. This chapter explains what each crate does, how they connect through ports and adapters, the shape of the configuration data, and how a single `lnix develop` invocation moves through the layers.

## Architecture Overview

The workspace is organized under `crates/`:

```
crates/
  lnix/          # Binary: CLI entry point (clap parsing + composition root)
  lnix-app/      # Library: use-cases (init/develop/run/test/task/lint/update/search)
  lnix-domain/   # Library: pure domain — definitions, services, ports, value objects
  lnix-infra/    # Library: adapters — filesystem, nix subprocess, nix-versions, stdout
```

The design follows a hexagonal ("ports and adapters") layout. `lnix-domain` is the innermost layer and performs no I/O. `lnix-app` orchestrates use-cases by talking only to traits (ports) that live in `lnix-domain`. `lnix-infra` supplies the concrete adapters, and the `lnix` binary is the composition root that wires everything together.

## Crates

### lnix

**Crate:** `lnix` (binary)
**Responsibility:** parse CLI arguments and act as the composition root.

`lnix` defines the `lnix` command and its subcommands with [clap](https://docs.rs/clap/):

| Subcommand | Description |
|------------|-------------|
| `init`     | Scaffold `lazynix.yaml` and starter files from the bundled templates |
| `update`   | Update `flake.lock` without entering the shell |
| `develop`  | Regenerate `flake.nix` and enter `nix develop` |
| `run`      | Run a single command inside the dev shell |
| `test`     | Run the `test` commands declared in `lazynix.yaml` |
| `task`     | Run a named task from the `task` section |
| `lint`     | Validate channel-based packages with `nix eval` |
| `search`   | Look up available versions via nix-versions |

The binary itself contains no business logic. `main.rs` parses arguments, constructs an `AdapterSet` (the composition root), borrows those adapters into an `lnix_app::Deps` bundle, and dispatches into the matching use-case in `lnix-app`.

### lnix-app

**Crate:** `lnix-app` (library)
**Responsibility:** orchestrate use-cases against domain ports.

Each subcommand maps to a function under `usecase/` shaped as `fn(&Deps, ...) -> Result<i32, ApplicationError>`. `Deps` is a borrowed bundle of every port a use-case may touch: `ConfigRepository`, `FlakeWriter`, `EnvFilePresenceChecker`, `ProjectScaffolder`, `NixRunner`, `NixEvaluator`, `VersionResolver`, and `OutputPort`.

The flake-generating use-cases (`develop`, `test`, `run`) share a common prefix defined in `pipeline.rs`:

1. `load_config` — read settings, read `lazynix.yaml`, run `validate_config` (diagnostics are surfaced via `OutputPort::warn`), check that referenced dotenv files exist, then resolve any pinned packages and persist the resolutions back into `lazynix.yaml`.
2. `write_flake` — call `lnix_domain::render_flake` and write the result to `./flake.nix`.
3. `maybe_update_lock` — call `NixRunner::flake_update` when `--update` was requested.

Errors compose on the railway: every step returns a `Result`, and focused domain errors are lifted into `ApplicationError` through `#[from]`, so use-case bodies stay linear and let `?` short-circuit failures up to `main()`.

### lnix-domain

**Crate:** `lnix-domain` (library)
**Responsibility:** the pure domain. No I/O; depends only on `serde` and `thiserror`.

Four sub-modules divide the domain:

- `definition/` — the configuration AST: `DevShellDefinition`, `DevShell`, `Package`, `PackageEntry`, `PinnedPackageEntry`, `Env`, `EnvVar`, `TaskDef`, `Settings`. Also `validate_config`, which produces `Diagnostic` values for non-fatal findings and returns `ValidationError` for hard failures.
- `values/` — value objects that validate their invariants at construction: `PackageName`, `PackageVersion`, `TaskName`, `EnvVarName`, `RegistryUrl`. These make illegal values unrepresentable in downstream code and double as the shell-injection defence for anything that flows into a generated Nix expression or a spawned subprocess.
- `service/` — pure domain services: `flake::render_flake` (turns a `DevShellDefinition` into a `flake.nix` string), `lint::*` (classifies raw `nix eval` errors and formats validation reports), `task::interpolate_command` (substitutes CLI arguments into task templates).
- `interface/` — the ports. Traits live under `interface::persistence` (`ConfigRepository`, `FlakeWriter`, `EnvFilePresenceChecker`, `ProjectScaffolder`), `interface::gateway` (`NixRunner`, `NixEvaluator`, `VersionResolver`), and `interface::output` (`OutputPort`).

### lnix-infra

**Crate:** `lnix-infra` (library)
**Responsibility:** concrete adapters for the domain ports.

Every trait declared in `lnix_domain::interface` gets an implementation here:

- `persistence/` — filesystem adapters (`ConfigRepository`, `FlakeWriter`, `EnvFilePresenceChecker`, `ProjectScaffolder`). All paths are anchored to `WorkspacePaths` so no adapter reads the current working directory implicitly.
- `gateway/` — subprocess adapters that call `nix` and `nix-versions`. Two private helpers (`run_inherit` for interactive commands, `run_capture` for evaluated output) keep stdio wiring and error mapping in one place.
- `output/` — the terminal sink that implements `OutputPort`.

`lnix-infra` also bundles the templates used by `lnix init`.

## Dependency Direction and Inversion

The dependency graph is a straight line with a single inversion:

```
lnix  ─►  lnix-app  ─►  lnix-domain  ◄─  lnix-infra
```

- `lnix` depends on `lnix-app` and `lnix-infra` only in the composition root.
- `lnix-app` depends only on `lnix-domain`. It never names a concrete adapter; it talks to trait objects (`&dyn ConfigRepository`, `&dyn NixRunner`, ...).
- `lnix-domain` depends on nothing internal. It defines the ports.
- `lnix-infra` depends on `lnix-domain` and implements its ports. The arrow points the "wrong" way on purpose: this is the **dependency inversion** that keeps the domain testable in isolation.

Because ports are traits, use-case tests substitute mocks by constructing `Deps` with `&dyn` references to fakes. Nothing about the use-case code changes between production and tests.

## Data Model

`DevShellDefinition` is the root of `lazynix.yaml`:

```
DevShellDefinition
  └── DevShell
        ├── allowUnfree:  bool                     (default: true)
        ├── package: Package
        │     ├── stable:   Vec<PackageEntry>              # { name: PackageName }
        │     ├── unstable: Vec<PackageEntry>              # { name: PackageName }
        │     └── pinned:   Vec<PinnedPackageEntry>        # { name, version, resolvedCommit?, resolvedAttr? }
        ├── shellHook:   Vec<String>
        ├── env:         Option<Env>                       # { dotenv: Vec<String>, envvar: Vec<EnvVar> }
        ├── test:        Vec<String>
        ├── task:        Option<HashMap<TaskName, TaskDef>>
        └── shellAlias:  Vec<String>                       # files whose alias definitions are loaded
```

Notes on the newer fields:

- `pinned` binds a package to an exact version. `resolvedCommit` and `resolvedAttr` are filled in the first time the pipeline resolves the version through `VersionResolver`, and are then written back to `lazynix.yaml` so subsequent runs skip the lookup.
- `shellAlias` lists files whose shell alias definitions are loaded into the dev shell.
- `env.envvar[].name` is an `EnvVarName` and `task` keys are `TaskName`, so invalid identifiers are rejected at YAML parse time.

There is no separate intermediate representation. `render_flake` walks `DevShellDefinition` directly to produce the `flake.nix` string.

## Validation Rules

`lnix_domain::validate_config` runs the cross-field checks that value objects cannot express. Field-level invariants — package name syntax, version non-emptiness, task and env-var name syntax — are already enforced when serde constructs the value objects, so `validate_config` only inspects the remaining relationships.

Two outcomes are possible:

- **Hard failure** — `ValidationError::EmptyTaskCommands(name)` when a task declares an empty `commands` list. The pipeline stops before rendering `flake.nix`.
- **Non-fatal diagnostic** — `Diagnostic::NoPackages` when `stable`, `unstable`, and `pinned` are all empty. `validate_config` returns it as data; `pipeline::load_config` forwards it to `OutputPort::warn` and execution continues.

Additional invariants are enforced elsewhere:

- `pipeline::validate_env_files` fails when `env.dotenv` references a file that does not exist.
- Value-object parsers (`PackageName`, `PackageVersion`, `TaskName`, `EnvVarName`) reject syntactically invalid input at parse time, which also serves as the shell-injection guard for anything that eventually flows into a generated Nix expression or a spawned subprocess.

## Data Flow

Here is what happens when a user runs `lnix develop`:

```
User runs: lnix develop [--update]
  │
  1. lnix (binary) parses arguments with clap.
  │
  2. lnix builds AdapterSet and borrows it into a lnix_app::Deps bundle,
     then dispatches to lnix_app::develop.
  │
  3. lnix_app::pipeline::load_config
       ├── ConfigRepository::read_settings         (optional lazynix-settings.yaml)
       ├── ConfigRepository::read_config           (lazynix.yaml → DevShellDefinition)
       ├── lnix_domain::validate_config            (diagnostics → OutputPort::warn)
       ├── validate_env_files                      (dotenv files must exist)
       └── resolve_pinned_packages                 (VersionResolver::resolve →
                                                    write back to lazynix.yaml)
  │
  4. lnix_app::pipeline::write_flake
       └── lnix_domain::render_flake → FlakeWriter::write_flake
  │
  5. lnix_app::pipeline::maybe_update_lock          (only when --update)
       └── NixRunner::flake_update
  │
  6. NixRunner::develop                             (execs `nix develop`,
                                                     replacing the current process)
```

Every step either succeeds and hands control to the next, or returns a `Result::Err` that is lifted into `ApplicationError` and printed by `main()`. There are no retries, no fallbacks, no hidden state.

## Summary

| Crate | Type | Responsibility |
|-------|------|---------------|
| `lnix`        | binary  | CLI parsing and composition root |
| `lnix-app`    | library | Use-cases and pipeline orchestration against ports |
| `lnix-domain` | library | Definitions, value objects, pure services, port traits |
| `lnix-infra`  | library | Adapters for the filesystem, `nix`, `nix-versions`, stdout |

Dependencies flow one way from `lnix` down to `lnix-domain`, with `lnix-infra` connecting to `lnix-domain` through the inverted arrow of the port traits. Each crate can be reasoned about, tested, and modified on its own.

## Note

Design documents that go deeper than this overview are maintained under `document/jp/design/` in Japanese only. See for example `document/jp/design/version-pinning.md` for the pinning workflow.
