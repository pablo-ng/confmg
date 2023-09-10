# confmg

Confmg allows you to have all your config files stored in a folder that is synced to a remote server (e.g. GitHub, Cloud Storage, ...) and selectively copy them to the right place on different platforms.

For example, you can have your `.gitconfig` file saved in the folder `~/.confmg`, which is under a private repository on GitHub. The same folder contains a `confmg.json` file that includes an entry with the label "git" and the respective file location (usually `~/.gitconfig`). Then, with only one command you can copy this `.gitconfig` file into it's right location. Other config files can have different locations depending on the platform but always the same content (e.g. VSCode configuration is under `~/.config/Code/User/settings.json` on Linux and `~/AppData/Roaming/Code/User/settings.json` on Windows).

## Installation

If you have the Rust toolchain installed you can simply run `cargo install confmg`. Otherwise you can download a binary from the [Releases](https://github.com/pablo-ng/confmg/releases) section.

## Configuration

The configuration for confmg is written in a JSON file. The default path is `~/.confmg/confmg.json`, but it can be overwritten with the `-c, --config-file` argument or the `CONFMG_CONFIG` environment variable. The config file should be written in the following structure:

```json
{
  "<Label for this config>": {
    "source": "<Path to the source config file relative to this file (usually located in the same folder)>",
    "targets": {
      "windows": "<Path to the target config file on Windows (optional)>",
      "linux": "<Path to the target config file on Linux (optional)>",
      "macos": "<Path to the target config file on MacOS (optional)>"
    }
  }
}
```

Here is an example confmg config file:

```json
{
  "git": {
    "source": ".gitconfig",
    "targets": {
      "linux": "~/.gitconfig",
      "macos": "~/.gitconfig",
      "windows": "~/.gitconfig"
    }
  },
  "bashrc": {
    "source": ".bashrc",
    "targets": {
      "linux": "~/.bashrc",
      "macos": "~/.bashrc"
    }
  },
  "vscode_settings": {
    "source": "vscode/settings.json",
    "targets": {
      "linux": "~/.config/Code/User/settings.json",
      "macos": "~/Library/Application Support/Code/User/settings.json",
      "windows": "~/AppData/Roaming/Code/User/settings.json"
    }
  }
}
```

## Use Cases and Usage

- When setting up a new PC you can download the remote confmg folder containing your config files and then run `confmg apply-source`. All config files then get copied to their right destination.
- When changing a config file on your local filesystem you can run `confmg diff <LABEL>` to see the changes and `confmg apply-target <LABEL>` to overwrite the file in the confmg folder with the changed one.
- Run `confmg edit-config` to open the confmg config file in your default editor.
- Run `confmg edit-source <LABEL>` or `confmg edit-target <LABEL>` to open the source or target file for a particular config in your default editor.
- Run `confmg --help` for full documentation.

## TODOs

- add the possibility to execute scripts
- use https://github.com/mitsuhiko/similar for diffs?
- add commands to add/edit/remove configs (see https://docs.rs/clap/latest/clap/_derive/_cookbook/typed_derive/index.html)
