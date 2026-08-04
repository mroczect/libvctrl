# libvctrl

[![Crates.io](https://img.shields.io/crates/v/libvctrl)](https://crates.io/crates/libvctrl)
[![Downloads](https://img.shields.io/crates/d/libvctrl?label=downloads)](https://crates.io/crates/libvctrl)
[![License: MIT](https://img.shields.io/crates/l/libvctrl)](#license)
[![Docs](https://docs.rs/libvctrl/badge.svg)](https://docs.rs/libvctrl)
[![CI](https://img.shields.io/github/actions/workflow/status/mroczect/libvctrl/rust.yml?branch=master)](https://github.com/mroczect/libvctrl/actions)
[![MSRV](https://img.shields.io/badge/MSRV-1.85.0-blue)](#installation)
[![LoC](https://img.shields.io/tokei/lines/github/mroczect/libvctrl)](https://github.com/mroczect/libvctrl)
[![Last Commit](https://img.shields.io/github/last-commit/mroczect/libvctrl)](https://github.com/mroczect/libvctrl/commits/master)
[![Repo Size](https://img.shields.io/github/repo-size/mroczect/libvctrl)](https://github.com/mroczect/libvctrl)

A robust, content-addressed version control engine for arbitrary data, designed
for embedding into applications.

libvctrl provides the core data model, storage abstractions, hashing, encoding,
commands, diffing, three-way merging, and cryptographic signing needed to build
version control functionality directly into applications -- without shelling out
to an external VCS or depending on a CLI tool. It is a library only and does
not ship a binary.
