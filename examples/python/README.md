# Python Example

## Pre-Requirements

[Nix](https://nixos.org/) is all you need. Install from [here (nixos.org/download)](https://nixos.org/download/).

## Create DevShell

You do not have to install LazyNix command. Just run following.

```
nix run github:shunsock/LazyNix -- develop # not installed lnix command
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
  allowUnfree: false
  package:
    stable:
      - python312
      - uv
      # add your favorite cli
    unstable: []
  shellHook:
    - "echo Python $(python --version) ready!"
    - "echo uv $(uv --version) ready!"
```


```shell
nix run github:shunsock/LazyNix -- run cli_you_added # not installed lnix command
lnix run cli_you_added # installed lnix command
```

### Tips

using alias is powerful. you do not need install manually.

```shell
alias lnix=nix run github:shunsock/LazyNix
```

## You want to use lnix command directly, right?

We've published Binaries for Linux and MacOS. You can install LazyNix manually by following commands.

### Linux (x86_64)

```shell
curl -L -o lnix https://github.com/shunsock/LazyNix/releases/latest/download/lnix-x86_64-linux
chmod +x lnix
$ sudo mv lnix /usr/local/bin/
```

### Linux (arm64)

```shell
curl -L -o lnix https://github.com/shunsock/LazyNix/releases/latest/download/lnix-aarch64-linux
chmod +x lnix
$ sudo mv lnix /usr/local/bin/
```

### MacOS (arm64)

```shell
curl -L -o lnix https://github.com/shunsock/LazyNix/releases/latest/download/lnix-aarch64-darwin
chmod +x lnix
$ sudo mv lnix /usr/local/bin/
```

