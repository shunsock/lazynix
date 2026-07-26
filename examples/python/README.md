# Python Example

## Pre-Requirements

[Nix](https://nixos.org/) is all you need. Install from [here (nixos.org/download)](https://nixos.org/download/).

## Create DevShell

You do not have to install LazyNix command. Just run following.

```
nix run github:shunsock/lazynix -- develop # not installed lnix command
lnix develop # installed lnix command
```

## Run your favorite cli

you can run any cli registered to Nix.
search cli from [search.nixos.org](https://search.nixos.org/packages).

```shell
vim lazynix.yaml # edit lazynix.yaml
```

```yaml
devShell:
  allowUnfree: true
  package:
    stable:
      - name: python312
      - name: uv
      # add your favorite cli
    unstable: []
    pinned: []
  shellHook:
    - "echo Python $(python --version) ready!"
    - "echo uv $(uv --version) ready!"
```


```shell
nix run github:shunsock/lazynix -- run cli_you_added # not installed lnix command
lnix run cli_you_added # installed lnix command
```

### Tips

using alias is powerful. you do not need install manually.

```shell
alias lnix=nix run github:shunsock/lazynix
```

## You want to use lnix command directly, right?

See [Installation](../../README.md#installation) in the repository README for
`nix profile install`, pre-built binaries, and build-from-source instructions.
