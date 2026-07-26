# Task and Test Example

This example demonstrates the two workflow primitives that `lazynix.yaml`
exposes at the top of `devShell`:

- `task:` — named workflows with optional `description` and `{{.CLI_ARGS}}`
  interpolation.
- `test:` — a flat list of commands to execute in order.

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
    pinned: []

  shellHook:
    - "echo 'Try: lnix task fmt / lnix task review <expr> / lnix test'"

  task:
    fmt:
      description: "Format Python sources with ruff"
      commands:
        - "uv run ruff format ."
    review:
      description: "Run a subset of pytest via CLI_ARGS"
      commands:
        - "uv run pytest {{.CLI_ARGS}}"

  test:
    - "uv run ruff check ."
    - "uv run pytest -q"
```

## Usage

Run the commands below from **this directory** (`examples/task-and-test`):

```bash
# Enter the dev shell
nix run github:shunsock/lazynix -- develop

# Run a named task (no trailing arguments)
nix run github:shunsock/lazynix -- task fmt        # runs `uv run ruff format .`

# Run a named task with CLI_ARGS forwarding
nix run github:shunsock/lazynix -- task review tests/test_api.py::test_auth
# expands to: uv run pytest tests/test_api.py::test_auth

# Run the whole test suite (both `test:` entries in order)
nix run github:shunsock/lazynix -- test
```

## `task:` vs `test:`

| Feature              | `task:`                                    | `test:`                          |
| -------------------- | ------------------------------------------ | -------------------------------- |
| Shape                | Named map (`name -> { description, commands }`) | Flat list of shell commands      |
| `description`        | Supported                                  | Not applicable                   |
| `{{.CLI_ARGS}}`      | Expanded from trailing CLI arguments       | Not expanded                     |
| Typical use case     | Developer workflows (fmt / lint / migrate) | CI or pre-commit "run everything" |

Pick `task:` when you want a discoverable, named workflow that may accept
free-form arguments. Pick `test:` when you want a single command that runs the
entire verification pipeline the same way on a developer machine and in CI.
