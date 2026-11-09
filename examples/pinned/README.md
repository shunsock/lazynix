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
to look up the nixpkgs commit that provides the requested version. That
commit comes back as a 7-digit short SHA, so `lazynix` expands it to the
40-digit full SHA via the Devbox v1 resolve API, then embeds the full SHA
into the generated `flake.nix` as its input URL. Subsequent runs re-encode
the same `(name, version)` with the same naming scheme used to write
`flake.nix` and match it against the existing `url` / `buildInputs` lines
by name — no decoding — so the pin remains reproducible even if the
upstream index changes. `lazynix.yaml` is never mutated.

## References

- Repository README: [Installation](../../README.md#installation)
- Design document (Japanese): [document/jp/design/version-pinning.md](../../document/jp/design/version-pinning.md)
