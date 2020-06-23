# dropignore

A tool to conveniently ignore files and folders from Dropbox sync.

![License Apache-2.0 or MIT](https://img.shields.io/badge/license-Apache--2.0%20or%20MIT-blue)
[![Build status](https://img.shields.io/github/workflow/status/mweirauch/dropignore/CI?logo=GitHub)](https://github.com/mweirauch/dropignore/actions?query=workflow%3ACI+branch%3Amaster)

## What this tool is and is not

`dropignore` allows you to ignore files and folders according to a set of matching patterns defined in a global configuration file which is living outside of your Dropbox folder.

As the selective sync feature got broken (see [Motivation](#motivation)) `dropignore` uses the [extended file system attributes](https://help.dropbox.com/de-de/files-folders/restore-delete/ignored-files) solution.

This is _currently_ not a [gitignore-style-solution everbody is begging for](https://www.dropboxforum.com/t5/Dropbox/Ignore-folder-without-selective-sync/idi-p/5926). I am still waiting for the current developments on Dropbox' side.

## Installation and Usage

You can download a binary release for Linux, macOS and Windows on the [Releases](https://github.com/mweirauch/dropignore/releases) page or compile and install it yourself in case you got a Rust installation set up:

```sh
cargo install --git https://github.com/mweirauch/dropignore
```

After you have installed the binary you need to create a configuration file which contains the ignore and skip specifications of the file or folder names you want to ignore (not sync) or skip (keep synced) with your Dropbox. Providing skip specifications is optional. You just need them in case a ignore specification is too broad and would include any files or folders you don't want to be ignored.

The configuration file locations are as follows:

| System      | Location                                                           |
| :---------- | :----------------------------------------------------------------- |
| Linux dist. | `/home/charly/.config/dropignore/dropignore.yml`                   |
| macOS       | `/Users/charly/Library/Preferences/dropignore/dropignore.yml`      |
| Windows     | `C:\Users\charly\AppData\Roaming\dropignore\config\dropignore.yml` |

The configuration file could look like this:

```yaml
matcher:
  ignore-specs:
    - pattern: "**/build"
    - pattern: "**/target"
  skip-specs:
    - pattern: "**/src/target"
```

Any skip-spec which matches always wins over a previous ignore-spec match. So with the previous configuration the folders `myproject/target` and `myproject/src/target` would be selected as ignore candidates but the skip-spec would only allow for the former to be actually ignored.

The supported glob patterns can be found in the [globset](https://docs.rs/globset) project.

> It is recommended to use the `-n` (dry-run) option when testing new ignore or skip specifications!

### One-time scanning

```sh
dropignore scan [-n] /path/to/Dropbox/
```

This will scan the given path (or the current working directory if omitted) for ignore candidates.

### Periodic watching

```sh
dropignore watch [-n] /path/to/Dropbox/
```

This will first perform a scan (see above) and then watch all subsequent file system changes and check for ignore candidates as they occur. Currently, these changes are handled after a delay of 2 seconds.

## Notes and Limitations

- **use at your own risk** - allthough no data deletion is performed, be warned.
- **only developed on Linux** - untested by myself on Windows and macOS (except integration tests)
- **using `.gitignore` as the source of exclusion patterns is currently not considered**
  - there might be _projects_ shared in Dropbox which are actually neither version controlled nor programming related
  - someone might like to gitgnore any IDE specific files or folders, but still keep them synced over Dropbox
  - global gitignore settings would need to be considered/sourced then as well

## Motivation

For years developers and artists use Dropbox to share projects. When working on these projects, build tools or other programs might create temporary output folders with a huge amount of files or size which shall not be synced to Dropbox.

The old _trick_ for not syncing them was to delete any content in the folder which shall be ignored once, waiting for the sync to finish and then selectively ignore (unselect) this folder in the selective sync settings.

Whenever this folder was re-created or filled with new content locally it was kept ignored and not synced to your Dropbox account.

In late 2019 Dropbox decided to [mess](https://www.dropboxforum.com/t5/Files-folders/How-to-manually-stop-sync-of-a-folder-but-still-retain-local/td-p/360922) [arround](https://www.dropboxforum.com/t5/Installs-integrations/Feedback-on-the-new-desktop-app-quot-ignore-files-quot-feature/td-p/380960) with the selective sync feature every creative got used to, ignoring files or folders got a mess.

Essentially the old _trick_ didn't work anymore. Whenever an ignored folder was locally deleted and re-created - e.g. the `target` folder for Maven - the Dropbox client immedeately renamed the local folder to `target (Selective Sync Conflict)` - effectively moving your build artifacts to where they are not found anymore. Whenever the original folder got re-created another copy would be placed next to the previously renamed folder.

So you either don't use the selective sync feature anymore and live with the fact that syncing your temporary output folders might take ages and consume quite some space in your Dropbox or you go with the [proposed solution to use extended file system attributes](https://help.dropbox.com/de-de/files-folders/restore-delete/ignored-files) - with the circumstance that this solution **does not work with re-created files or folders which are to be ignored** as the attributes are gone after deletion.

So `dropignore` as a first-time Rust project for me came to be.

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
