# Philosophy

LazyNix exists because reproducible development environments should not require learning a new programming language. This chapter explains the design decisions that shape LazyNix and the trade-offs they involve.

## The Problem

Setting up a development environment is one of the most frustrating parts of joining a project. README files say "install Python 3.12, Node 18, and PostgreSQL", but they rarely say which minor version, which build flags, or which platform-specific quirks to watch out for. The result is a familiar ritual: a new team member spends hours debugging environment differences before writing a single line of application code.

Nix solves this problem at a fundamental level. A Nix flake can declare every dependency with cryptographic precision, ensuring that every developer gets a bit-for-bit identical environment. But Nix introduces a different problem: its own learning curve. The Nix language is a lazy, purely functional language with unfamiliar syntax. For a team that just wants "everyone gets the same Python", learning Nix feels like overkill.

LazyNix bridges this gap.

## YAML-Only Configuration

The core design decision in LazyNix is that users never write Nix. Instead, they write a `lazynix.yaml` file that declares what they need:

```yaml
devShell:
  package:
    stable:
      - python312
      - uv
    unstable: []
```

LazyNix translates this into a valid `flake.nix` behind the scenes. The user does not need to understand Nix expressions, attribute sets, or the nixpkgs module system. They only need to know the names of the packages they want, which they can look up at [search.nixos.org](https://search.nixos.org/packages).

This is a deliberate trade-off. LazyNix sacrifices the full expressiveness of Nix in exchange for an interface that any developer can read and modify in seconds. A YAML file is self-evident. A Nix expression, for the uninitiated, is not.

### Leveraging Existing Tools

LazyNix does not invent a new ecosystem. It leverages tools that already exist:

- **nixpkgs** provides the packages. LazyNix does not maintain its own package repository.
- **Nix flakes** provide the reproducibility guarantees. LazyNix generates standard flakes.
- **nix develop** provides the shell environment. LazyNix invokes it directly.

By building on existing infrastructure rather than replacing it, LazyNix remains a thin translation layer. When nixpkgs adds a new package, LazyNix users get access to it immediately without any changes to LazyNix itself.

### Simplifying the Interface

LazyNix reduces the surface area a developer needs to understand. Consider what a developer must know with each approach:

**Without LazyNix (pure Nix):**
- The Nix language syntax
- How to write `flake.nix` inputs and outputs
- The `mkShell` function and its arguments
- How `nixpkgs` attribute paths work
- How to pin package versions with `flake.lock`

**With LazyNix:**
- YAML syntax
- Package names from search.nixos.org

Everything else is handled by LazyNix's code generation.

## DevShell Only

LazyNix does one thing: create development shell environments. It does not build packages. It does not define NixOS system configurations. It does not manage deployment infrastructure.

This constraint is intentional. Nix is a vast ecosystem that can manage everything from development tools to production servers. But the use case with the highest return on investment for most teams is the development environment. Getting every developer onto the same toolchain eliminates an entire category of bugs and wasted time.

By focusing on this single use case, LazyNix can keep its configuration surface small and its error messages specific. When something goes wrong, the problem is almost always "this package name does not exist" or "this package is not available for your platform" --- problems that `lnix lint` catches before they reach the user.

## The Escape Hatch

LazyNix generates a standard `flake.nix` file. This is not an implementation detail --- it is a design guarantee. At any point, a team can stop using LazyNix and continue with the generated `flake.nix` directly.

The migration is two steps:

1. Delete `lazynix.yaml`
2. Edit `flake.nix` by hand from now on

No rewiring, no export process, no data loss. The generated `flake.nix` is a complete, standalone Nix flake that works without LazyNix installed.

This escape hatch matters because teams grow. A project that starts with "we just need Python and a linter" may eventually need custom Nix overlays, cross-compilation, or integration with a larger Nix infrastructure. When that day comes, LazyNix steps aside gracefully rather than becoming a constraint.

The design philosophy here is that **a tool should never trap its users**. LazyNix lowers the barrier to entry for Nix. When users outgrow that barrier, they graduate to full Nix without penalty.

## What LazyNix Is Not

To clarify LazyNix's scope, here is what it explicitly does not aim to be:

- **A Nix replacement.** LazyNix generates Nix. It does not replace or abstract away Nix at runtime.
- **A package manager.** LazyNix does not install packages. Nix does. LazyNix just tells Nix which packages to include.
- **A build system.** LazyNix does not compile your code, produce artifacts, or manage build pipelines.
- **A system configuration tool.** LazyNix does not manage NixOS modules, Home Manager, or system-level settings.

By being clear about what LazyNix is not, users can make an informed decision about when to use it and when to reach for a different tool.

## Summary

LazyNix is built on three principles:

1. **Configuration through declaration.** Users declare what they need in YAML. LazyNix handles the how.
2. **Focused scope.** Development shell environments only. Do one thing well.
3. **No lock-in.** The generated `flake.nix` is the escape hatch. Users can leave at any time.

These principles keep LazyNix simple, predictable, and honest about its limitations.
