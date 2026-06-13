# LazyNix crates

LazyNix is a Cargo workspace.

The layout follows [astral-sh/uv](https://github.com/astral-sh/uv/tree/main/crates):
one binary crate on top, with focused library crates underneath.

This file is the index.

Each crate has its own README with more detail.

## Crates

| Crate | Kind | Responsibility |
|-------|------|----------------|
| [`lnix`](./lnix) | binary | CLI entry point and command orchestration. |
| [`lnix-core`](./lnix-core) | library | Domain value objects and the `lazynix.yaml` config AST. |
| [`lnix-flake-generator`](./lnix-flake-generator) | library | Parse `lazynix.yaml` and render `flake.nix`. |
| [`lnix-linter`](./lnix-linter) | library | Validate packages exist via `nix eval`. |
| [`lnix-nix-dispatcher`](./lnix-nix-dispatcher) | library | Run `nix` commands as subprocesses. |

## Dependency direction

Dependencies flow in one direction.

The binary depends on the libraries.

The libraries depend on `lnix-core` (or nothing of ours).

```
                 lnix  (binary)
        ┌──────────┼───────────┬───────────────────┐
        v          v           v                   v
 flake-generator  linter   nix-dispatcher          │
        │          │                               │
        └────┬─────┘                               │
             v                                     │
          lnix-core  <───────────────(no dep)──────┘
```

`lnix-core` is the foundation.

It performs no I/O and depends only on `serde` and `thiserror`.

`lnix-nix-dispatcher` is independent.

It does not depend on any other LazyNix crate.

## Why this shape

Domain types live in one place (`lnix-core`).

So validation rules are not duplicated across crates.

Each library has a single responsibility.

So a change to how Nix is invoked does not touch how YAML is parsed.

The dependency graph has no cycles.

So each crate can be understood, tested, and changed on its own.
