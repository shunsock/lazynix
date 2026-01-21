<p align="center">
  <img width="512" height="256" alt="LazyNix Logo" src="https://github.com/user-attachments/assets/d3995d96-0ee4-45ce-9063-5d0c39864504" />
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

Download platform-specific binaries from [GitHub Releases](https://github.com/shunsock/LazyNix/releases).

#### 🐧 Linux x86_64

```bash
curl -L -o lnix https://github.com/shunsock/LazyNix/releases/latest/download/lnix-x86_64-linux
chmod +x lnix
sudo mv lnix /usr/local/bin/
```

#### 🐧 Linux ARM64

```bash
curl -L -o lnix https://github.com/shunsock/LazyNix/releases/latest/download/lnix-aarch64-linux
chmod +x lnix
sudo mv lnix /usr/local/bin/
```

#### 🍎 macOS Apple Silicon

```bash
curl -L -o lnix https://github.com/shunsock/LazyNix/releases/latest/download/lnix-aarch64-darwin
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
  allowUnfree: false

  package:
    stable:
      - python312
      - uv
    unstable: []

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

