# hell

[![Build Status](https://travis-ci.org/hellboxpy/hell.svg?branch=master)](https://travis-ci.org/hellboxpy/hell)

The command line interface (CLI) for the [hellbox framework](https://github.com/hellboxpy/hellbox).

## Installation

The `hell` binary can be installed by running the `install.sh` script from the Hellbox website:

```shell
curl https://www.hellbox.io/install.sh -sSf | sh
```

This will download the pre-compiled binary for your computer's architecture and install it into `/usr/bin/local`. Mac OS and some linux distributions are currently supported (with Windows support to come).

## Commands

### `hell init`

Sets up a new project by:

* Creating a new Python 3 virtual environment
* Installing `hellbox` and creating a `pyproject.toml`
* Creating a minimal `Hellfile.py` for defining tasks

### `hell run {task}`

Runs the task defined in `Hellfile.py`. Defaults to the task named `default`.

### `hell install`

Installs all packages in `pyproject.toml` into the project's Python installation.

### `hell add {package}`

Installs a package using `uv` into the project's Python installation

### `hell remove {package}`

Uninstalls a package using `uv` from the project's Python installation

### `hell inspect`

Loads the `Hellfile.py` manifest and displays the defined tasks:

```
│ » build
│   Builds font files from source
╽
┗━ ReadFiles(globs=('source/*.ufo',))
   ┣━ GenerateOtf
   ┃  ┗━ RenameFile(prefix='16th-Century-Gothic-')
   ┃     ┗━ DummyDsig
   ┃        ┗━ Autohint
   ┃           ┗━ Write(path='build/otf')
   ┗━ GenerateTtf
      ┗━ RenameFile(prefix='16th-Century-Gothic-')
         ┗━ DummyDsig
            ┗━ Autohint
               ┣━ GenerateWoff2
               ┃  ┗━ Write(path='build/woff2')
               ┗━ Write(path='build/ttf')
```
