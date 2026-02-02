# Denv

`denv` is a personal collection of shell environment files, configuration 
snippets and utility scripts used to bootstrap Linux or macOS systems.

The project provides an install script that copies configuration files to 
`$HOME`, installs dependencies and generates a single `/etc/envrc` file from 
the contents of the `env/` directory. This file can then be sourced from your 
shell to load aliases, functions and environment variables.

## Repository layout

```
./install         - Install packages and run `deploy` to set up the environment
./deploy          - Copies files from this repo to `~/.config`, `~/.local/bin` and merges env scripts
env/              - Shell environment fragments (global settings, git aliases, etc.)
common/.config    - User configuration files (tmux, ghostty, backgrounds)
common/.local/bin - Helper scripts (tmux sessionizer, network tools, etc.)
rc/               - Additional dotfiles copied to `$HOME`
```

The directories `linux/` and `mac/` are reserved for platform specific
configuration which can be placed under `.config` or other paths if needed.

## Usage

Run the install script from the repository root:

```bash
./install
```

The script detects the platform and installs required packages via `apt-get` or
`brew`.

Once completed you can source `/etc/envrc` from your shell (`.bashrc`, `zshrc`,
etc.) to enable the provided aliases and functions:

```bash
. /etc/envrc
```

Utility scripts placed in `common/.local/bin` will be available in your `$PATH`
after running the install process.

## License

This project is distributed under the terms of the MIT License. See the
[LICENSE](LICENSE) file for details.

