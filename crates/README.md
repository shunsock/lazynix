# LazyNix crates

LazyNix is a Cargo workspace.

The layout follows clean architecture: one crate per layer, one binary on top.

This file is the index.

Each crate has its own README with more detail.

## Crates

| Crate | Kind | Responsibility |
|-------|------|----------------|
| [`lnix`](./lnix) | binary | CLI parsing (clap) and the composition root that wires adapters. |
| [`lnix-app`](./lnix-app) | library | Application layer: use-cases, dependency bundle (`Deps`), and `ApplicationError`. |
| [`lnix-domain`](./lnix-domain) | library | Domain layer: value objects, the `DevShellDefinition` AST, pure services, and ports. |
| [`lnix-infra`](./lnix-infra) | library | Infrastructure layer: filesystem, subprocess, and terminal adapters for the domain ports. |

## Dependency direction

Dependencies flow inward, in one direction.

```
              lnix  (binary)
               ┌───┴────┐
               v        v
           lnix-app  lnix-infra
               └───┬────┘
                   v
              lnix-domain
```

`lnix-domain` is the innermost layer.

It performs no I/O and depends only on `serde` and `thiserror`.

It owns the ports (interfaces) that the outer layers implement.

`lnix-app` is the application layer: use-cases over `&dyn` ports.

`lnix-infra` is the infrastructure layer: the port implementations.

Both depend only on `lnix-domain`.

The binary constructs the adapters once and lends them to use-cases.

## Why this shape

The crate boundary is the layer boundary.

So a dependency in the wrong direction is a compile error, not a review comment.

Use-cases touch I/O only through ports.

So every command is unit-tested with mocks: no filesystem, no `nix`, no terminal.

All `nix` subprocess execution lives in one place (`lnix-infra`'s gateways).

So how `nix` is invoked can change without touching what commands do.
