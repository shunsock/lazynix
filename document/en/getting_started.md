# Getting Started

Welcome to LazyNix! This guide will walk you through setting up your first reproducible development environment. By the end, you will have a working project where every team member gets the exact same tools, at the exact same versions, with a single command.

No prior Nix knowledge is required. LazyNix handles the Nix complexity behind the scenes so you can focus on what matters: writing code.

## Installing Nix

LazyNix builds on top of [Nix](https://nixos.org/), a package manager that guarantees reproducible builds. Before using LazyNix, you need Nix installed on your machine.

Visit [nixos.org/download](https://nixos.org/download/) and follow the instructions for your operating system. Once installed, verify it works:

```bash
nix --version
```

You should see something like `nix (Nix) 2.x.x`.

LazyNix uses Nix Flakes, which requires the `flakes` and `nix-command` experimental features to be enabled. Add the following to your Nix configuration file (`~/.config/nix/nix.conf`):

```
experimental-features = nix-command flakes
```

If the file does not exist, create it. After saving, run `nix --version` again to confirm Nix still works. That is all the Nix setup you need. LazyNix takes care of the rest.

## Installing LazyNix

The simplest way to try LazyNix is to run it directly from GitHub without installing anything:

```bash
nix run github:shunsock/lazynix -- --help
```

For everyday use, install LazyNix to your Nix profile so the `lnix` command is always available:

```bash
nix profile install github:shunsock/lazynix
```

Verify the installation:

```bash
lnix --help
```

> **Note:** If you prefer not to install, you can always substitute `lnix` with `nix run github:shunsock/lazynix --` in all commands throughout this guide.

## Creating Your First Project

Navigate to your project directory (or create a new one) and run:

```bash
lnix init
```

This creates two files:

- `lazynix.yaml` --- your environment configuration. This is the only file you edit.
- `flake.nix` --- a generated Nix flake. LazyNix manages this for you.

Let's look at the generated `lazynix.yaml`:

```yaml
devShell:
  allowUnfree: false
  package:
    stable:
      - hello
    unstable: []
  shellHook:
    - "echo Welcome to LazyNix DevShell!"
```

This is the entire configuration. Let's break down what each field means:

- **`allowUnfree`** --- Nix distinguishes between open-source and proprietary (unfree) packages. Set this to `true` if you need proprietary software like VS Code or CUDA toolkits. When set to `false`, only open-source packages are allowed.
- **`package.stable`** --- Packages sourced from a stable, well-tested snapshot of [nixpkgs](https://github.com/NixOS/nixpkgs) (the Nix package repository). Use this for most tools.
- **`package.unstable`** --- Packages sourced from the latest nixpkgs. Use this when you need a cutting-edge version that has not yet reached the stable channel.
- **`shellHook`** --- Shell commands that run automatically every time you enter the development environment. Useful for printing version info, setting up aliases, or running initialization scripts.

There is no Nix syntax to learn --- just YAML.

## Customizing Your Environment

Suppose you are starting a Python project with [uv](https://docs.astral.sh/uv/) as the package manager. Open `lazynix.yaml` and replace its contents:

```yaml
devShell:
  allowUnfree: false
  package:
    stable:
      - python312
      - uv
    unstable: []
  shellHook:
    - "echo Python $(python --version) ready!"
    - "echo uv $(uv --version) ready!"
```

We put `python312` and `uv` under `package.stable` because these are well-established tools where the stable channel provides exactly what we need. The shell hooks confirm the tools are available each time you enter the environment.

How do you find the right package names? Visit [search.nixos.org](https://search.nixos.org/packages) and search for the tool you need. The package name shown there is exactly what you put in `lazynix.yaml`.

## Entering the Development Environment

Now, enter your configured environment:

```bash
lnix develop
```

LazyNix reads your `lazynix.yaml`, generates a fresh `flake.nix` (overwriting any existing one), and drops you into a shell where Python 3.12 and uv are available. You should see output like:

```
Reading configuration from .
Validating configuration...
Generating flake.nix...
✓ flake.nix generated successfully
Skipping flake.lock update (use --update to update)

Python Python 3.12.x ready!
uv uv 0.x.x ready!
```

Every developer on your team who runs `lnix develop` gets the same Python version, the same uv version, every time. No more "works on my machine" problems.

To update pinned package versions, add the `--update` flag:

```bash
lnix develop --update
```

This refreshes `flake.lock` --- a lock file that pins the exact version of every package. Without `--update`, LazyNix uses the versions recorded in `flake.lock`, ensuring that every developer gets identical tools. With `--update`, it pulls in the latest versions from nixpkgs.

> **Important:** `lnix develop` regenerates `flake.nix` from `lazynix.yaml` every time it runs. If you have manually edited `flake.nix`, those changes will be overwritten. See [Philosophy](./philosophy.md) for more on this design decision and how to migrate to pure Nix when you are ready.

## Running Commands

Sometimes you do not need a full interactive shell. You just want to run a single command inside the environment. That is what `lnix run` is for:

```bash
lnix run -- python -c "print('Hello from LazyNix!')"
```

The `--` separates LazyNix flags from the command you want to run. Everything after `--` is executed inside the Nix development environment.

## Defining Tasks

For commands you run repeatedly, you can define named tasks in `lazynix.yaml`:

```yaml
devShell:
  allowUnfree: false
  package:
    stable:
      - python312
      - uv
    unstable: []
  shellHook:
    - "echo Python $(python --version) ready!"

  task:
    test:
      description: "Run the test suite"
      commands:
        - "python -m pytest"
    fmt:
      description: "Format code"
      commands:
        - "uv run ruff format ."
```

Then run a task by name:

```bash
lnix task test
lnix task fmt
```

Tasks run sequentially inside the development environment, so all your declared packages are available.

## Running Tests

If your project has test commands, you can define them directly in `lazynix.yaml` using the `test` field:

```yaml
devShell:
  package:
    stable:
      - python312
      - uv
    unstable: []

  test:
    - "python -m pytest"
    - "python -m mypy src/"
```

Then run all tests with:

```bash
lnix test
```

This enters the development environment and executes each test command sequentially. If any command fails, `lnix test` exits with a non-zero status code.

The difference between `test` and `task` is intent: `test` is a flat list of commands designed for CI pipelines and pre-commit checks, while `task` defines named, reusable workflows with descriptions and argument interpolation.

## Validating Your Configuration

Before committing your configuration, you can verify that all declared packages actually exist in nixpkgs:

```bash
lnix lint
```

This checks each package using `nix eval` and reports any that cannot be found. It catches typos and non-existent package names before they cause a confusing error at build time.

## What You Have Learned

In this guide, you have:

- Installed Nix, the only prerequisite
- Created a LazyNix project with `lnix init`
- Configured a Python development environment in `lazynix.yaml`
- Entered the environment with `lnix develop`
- Run commands with `lnix run` and defined reusable tasks
- Validated your configuration with `lnix lint`

## Next Steps

- Read [Philosophy](./philosophy.md) to understand the design decisions behind LazyNix
- Read [System Architecture](./system_architecture.md) to learn how LazyNix is built internally
- Explore the [examples](../../examples/) directory for more configuration patterns
