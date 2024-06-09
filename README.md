# update-pypi-deps

master: [![master branch build status](https://github.com/n8henrie/update-pypi-deps/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/n8henrie/update-pypi-deps/actions/workflows/ci.yml)

Parse pypi dependencies from pyproject.toml and output the latest versions

[![Crates.io](https://img.shields.io/crates/v/update-pypi-deps.svg)](https://crates.io/crates/update-pypi-deps)
[![Docs.rs](https://docs.rs/update-pypi-deps/badge.svg)](https://docs.rs/update-pypi-deps)

NB: This is a low-investment toy / hobby project.

## Features

1. Parses a `pyproject.toml`
2. Pulls out the `dependencies` and `optional-dependencies`
3. Concurrently fetches the default pypi release (usually but not always the
   latest version) for each dependency in each of these
4. Prints this out in a format easy to copy and paste back into your pyproject.toml

## Introduction

Sometimes I want to update all of a python's project top-level dependencies to
the latest version and see if it still works. If so, I commit the change. If
not, I try to sort out which of the dependencies are holding me back and why.
Most sane people will just use something like poetry or pip-tools, but after
getting burned by investing time to learn Pipenv, I now prefer to do it the old
fashioned way.

## Quickstart

```console
$ update-pypi-deps [ -i /path/to/pyproject.toml ]
```

### Cargo

* Install the rust toolchain in order to have cargo installed by following
  [this](https://www.rust-lang.org/tools/install) guide.
* run `cargo install update-pypi-deps`

### nix

```console
$ nix run github:n8henrie/update-pypi-deps
```

## Troubleshooting / FAQ

- This currently doesn't handle complex version constraints (like
`fauxmo>0.1,<0.6`); it just reuses the same contraint it is given, but with a
new version number

