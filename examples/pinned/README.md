# Pinned Example

This example demonstrates how to combine **stable** packages with a **pinned**
package to build a dev shell that includes a specific version of a tool
(here: Go `1.21.13`) that is no longer available in the current stable channel.

## Pre-Requirements

[Nix](https://nixos.org/) is all you need. Install from [nixos.org/download](https://nixos.org/download/).

For installing the `lnix` command permanently, see [Installation](../../README.md#installation).

## Configuration

```yaml
devShell:
  allowUnfree: true
  package:
    stable:
      - name: python312
      - name: uv
    unstable: []
    pinned:
      - name: go
        version: "1.21.13"
  shellHook:
    - "echo Python $(python --version) ready!"
    - "echo Go $(go version) ready!"
```

## Usage

Run the commands below from **this directory** (`examples/pinned`):

```bash
# Explore which nixpkgs commit provides go 1.21.x
nix run github:shunsock/lazynix -- search go -v '>=1.21,<1.22'

# Enter the dev shell (resolves and pins go 1.21.13 on first run)
nix run github:shunsock/lazynix -- develop
```

## Note: Resolution Caching

When you invoke `lnix develop` and no `flake.nix` exists, `lazynix` uses
[`nix-versions`](https://lazamar.github.io/download-specific-package-version-with-nix/)
to look up the exact nixpkgs commit that provides the requested version, then
embeds that commit into the generated `flake.nix` as its input URL. Subsequent
runs reuse the commit baked into `flake.nix` and skip the resolver call, so
the pin remains reproducible even if the upstream index changes.
`lazynix.yaml` is never mutated.

## References

- Repository README: [Installation](../../README.md#installation)
- Design document (Japanese): [document/jp/design/version-pinning.md](../../document/jp/design/version-pinning.md)
