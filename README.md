<p align="center">
  <img width="512" height="256" alt="LazyNix Logo" src="./logo.png" />
</p>
<p align="center">
  <em>Providing reproducible environments for all developers</em>
</p>

## Why LazyNix?

LazyNix makes Nix development environments accessible through simple YAML configuration.

- 🚀 **Simple**: Write YAML instead of Nix expressions
- 🔄 **Reproducible**: Powered by Nix flakes for deterministic builds
- 🎯 **Focused**: Designed for DevShell only - when you need more, just use the generated `flake.nix`

## Installation

### 📋 Pre-Requirements

We recommend using [Nix](https://nixos.org/) to install LazyNix. If you don't have Nix installed yet, get it from [nixos.org/download](https://nixos.org/download/).

Alternatively, you can use pre-built binaries from the release page or build from source.

### ⚡ No Installation Required

Try LazyNix without installing anything. Run it directly from GitHub using `nix run`:

```bash
# Display help
nix run github:shunsock/lazynix -- --help

# Initialize a new project
nix run github:shunsock/lazynix -- init

# Enter development environment
nix run github:shunsock/lazynix -- develop
```

### ❄️ Install to Profile

For permanent installation, add LazyNix to your Nix profile:

```bash
# Install from GitHub
nix profile install github:shunsock/lazynix

# Then use the lnix command directly
lnix --help
lnix init
lnix develop
```

### 📦 Pre-built Binaries

Download platform-specific binaries from [GitHub Releases](https://github.com/shunsock/lazynix/releases).

#### 🐧 Linux x86_64

```bash
curl -L -o lnix https://github.com/shunsock/lazynix/releases/latest/download/lnix-x86_64-linux
chmod +x lnix
sudo mv lnix /usr/local/bin/
```

#### 🐧 Linux ARM64

```bash
curl -L -o lnix https://github.com/shunsock/lazynix/releases/latest/download/lnix-aarch64-linux
chmod +x lnix
sudo mv lnix /usr/local/bin/
```

#### 🍎 macOS Apple Silicon

```bash
curl -L -o lnix https://github.com/shunsock/lazynix/releases/latest/download/lnix-aarch64-darwin
chmod +x lnix
sudo mv lnix /usr/local/bin/
```

### 🔨 Build from Source

Clone the repository and build using Nix:

```bash
# Clone the repository
git clone https://github.com/shunsock/lazynix.git
cd lazynix

# Build with Nix
nix build

# Run the built binary
./result/bin/lnix --help
```

## Quick Start

### Initialize a New Project

Create a new LazyNix configuration in your project directory:

```bash
lnix init
```

This creates two files:
- 📝 `lazynix.yaml` - Your environment configuration (edit this)
- ⚙️ `flake.nix` - Generated Nix flake (auto-generated, don't edit)

### Configure Your Environment

Edit `lazynix.yaml` to specify your development tools. Find packages at [search.nixos.org](https://search.nixos.org/packages).

```yaml
devShell:
  allowUnfree: true

  package:
    stable:
      - name: python312
      - name: uv
    unstable: []
    pinned: []

  shellHook:
    - "echo Python $(python --version) ready!"
    - "echo uv $(uv --version) ready!"

  env:
    # Load from .env files
    dotenv:
      - .env

    # Define variables directly
    envvar:
      - name: PYTHONPATH
        value: ./src
      - name: DEBUG
        value: "true"
```

### Enter the Development Environment

Activate your configured environment:

```bash
lnix develop
```

LazyNix will automatically:
1. 📖 Read your `lazynix.yaml` configuration
2. 🔧 Generate the `flake.nix` file
3. 🔒 Update `flake.lock` with pinned dependencies (with `--update`)
4. 🚀 Enter the Nix development shell with all specified packages

## Commands Reference

LazyNix ships eight subcommands. All commands accept the global flags
described below.

| Subcommand | Description | Flags |
|-----------|-------------|-------|
| `init` | Create `lazynix.yaml` and `flake.nix` from templates | `--force` (`-f`) — overwrite existing files |
| `update` | Update `flake.lock` without entering a shell | — |
| `develop` | Generate `flake.nix` and enter `nix develop` | `--update` — update `flake.lock` first |
| `run [--] <command>...` | Run a single command inside the dev environment | `--update`, `--no-regen` (skip regenerating `flake.nix`) |
| `test` | Run test commands defined under `devShell.test:` | `--update` |
| `task <name> [args...]` | Run a named task from `devShell.task:`; trailing args expand into `{{.CLI_ARGS}}` | — |
| `lint` | Validate every stable/unstable package via `nix eval` | `--verbose` (`-v`), `--arch <target>` |
| `search <package>` | Look up available versions via `nix-versions` | `--version <semver>` (`-v`), `--json` (`-j`), `--one` (`-1`) |

### Global Flags

- `-C, --config-dir <DIR>` — directory containing `lazynix.yaml` and
  `lazynix-settings.yaml` (env: `LAZYNIX_CONFIG_DIR`, default: current
  directory)
- `--version` — print the CLI version and exit

### Exit Codes and Notes

- `lint` exits with code `1` when any package fails validation, otherwise `0`.
- `run`, `task`, and `test` propagate the exit code of the underlying
  child process.
- `lint` deliberately **excludes `pinned` packages from validation** —
  only entries under `stable` and `unstable` are evaluated. Pinned
  packages are checked at resolution time by `search` via
  `nix-versions` instead.

## Configuration

### Custom Config Directory

By default, LazyNix looks for `lazynix.yaml` and `lazynix-settings.yaml` in the current directory. You can customize this location using either a CLI flag or environment variable.

#### Methods

**1. CLI Flag (Recommended for one-off usage)**

Use the `--config-dir` flag (or `-C` short form) before the subcommand:

```bash
lnix --config-dir ./configs develop
lnix -C ./configs develop  # Short form
```

**2. Environment Variable (Recommended for persistent setup)**

Set the `LAZYNIX_CONFIG_DIR` environment variable:

```bash
LAZYNIX_CONFIG_DIR=./configs lnix develop

# Or export for the entire session
export LAZYNIX_CONFIG_DIR=./configs
lnix init
lnix develop
```

## Advanced Configuration

### 📋 Settings File (Optional)

LazyNix supports an optional `lazynix-settings.yaml` file for system-level customization. This file is completely optional - LazyNix works perfectly without it using sensible defaults.

**When to use settings:**
- Override nixpkgs versions (use older/newer packages)

### 🎛️ Override Stable Nixpkgs

By default, LazyNix uses `nixos-25.11` for stable packages. You can override this in `lazynix-settings.yaml`:

```yaml
# lazynix-settings.yaml
override-stable-package: "github:myorg/nixpkgs/custom-branch"
```

`override-stable-package` only affects the **stable** channel; the
unstable channel is hardcoded to `github:NixOS/nixpkgs/nixos-unstable`.

### 📌 Version Pinning

The `devShell.package.pinned` list lets you pin a package to an exact
version, resolved through `nix-versions` and locked into the generated
flake. This is the recommended way to control language runtimes such
as `go`, `node`, or `python` down to the patch level.

Workflow:

1. Find a candidate version:

   ```bash
   lnix search go -v '>=1.21,<1.22'
   ```

2. Add the resolved name and version to `lazynix.yaml`:

   ```yaml
   devShell:
     package:
       stable:
         - name: python312
       pinned:
         - name: go
           version: "1.21.13"
   ```

3. Run `lnix develop` (or `run`/`test`). LazyNix resolves the pinned
   entry via `nix-versions`, populates `resolvedCommit` and
   `resolvedAttr` in the same list entry, and the resolved values are
   **written back to `lazynix.yaml`** automatically:

   ```yaml
   devShell:
     package:
       pinned:
         - name: go
           version: "1.21.13"
           resolvedCommit: "5ed6275"
           resolvedAttr: "go_1_21"
   ```

   Subsequent commands reuse the already-resolved entry and skip the
   network round-trip.

### 🔤 Shell Aliases

Alias definitions can be sourced from external files via
`devShell.shellAlias`. Each entry is a path — relative, absolute, or
`~`-prefixed — to a shell script whose alias definitions will be
loaded into the dev shell.

```yaml
devShell:
  allowUnfree: true
  package:
    stable:
      - name: bash
  shellAlias:
    - ./aliases.sh
    - ~/.bash_aliases
    - /etc/aliases.sh
```

Relative paths are resolved against `$PWD`; `~` is expanded to the
user's home directory; absolute paths are used as-is.

### 🧩 Tasks and Tests

Two related sections describe reusable commands that run inside the
dev shell:

- `devShell.test` — a flat list of shell commands. `lnix test` runs
  them in order and stops on the first failure. Use this for smoke
  tests you want to run without remembering a task name.
- `devShell.task` — a named map of workflows, each with an optional
  `description` and a list of `commands`. Run a task with
  `lnix task <name>`. Any trailing arguments are substituted into the
  `{{.CLI_ARGS}}` placeholder inside the task's commands, so a single
  task can accept variable arguments.

```yaml
devShell:
  allowUnfree: true
  package:
    stable:
      - name: python312
      - name: uv

  task:
    fmt:
      description: "Format Python sources"
      commands:
        - "uv run ruff format ."
    review:
      description: "Run a specific pytest, forwarded via CLI_ARGS"
      commands:
        - "uv run pytest {{.CLI_ARGS}}"

  test:
    - "uv run pytest"
    - "uv run mypy src/"
```

Example invocations:

```bash
lnix task fmt
lnix task review tests/test_api.py::test_auth   # expands into {{.CLI_ARGS}}
lnix test
```

## Design Philosophy

### ✅ What LazyNix Does

- **Reproducible Development Environments**: Consistent, shareable dev setups
- **Simple Configuration Interface**: YAML instead of Nix expressions

### ❌ What LazyNix Doesn't Do

- **Cover All Nix Features**: No build definitions, overlays, or modules
- **Replace Nix**: It's a thin layer on top of Nix flakes
- **Manage System Configuration**: Only development environments

## Migration from LazyNix to Pure Nix

When you need advanced Nix features, migration is seamless. LazyNix generates a standard `flake.nix`, so:

1. 🗑️ Delete `lazynix.yaml`
2. ✏️ Continue editing `flake.nix` directly

That's all! Your development environment keeps working without any changes.

## Contribution

We welcome contributions!

## License

This project is licensed under the [MIT License](./LICENSE).

