# mvr

[![Travis Build Status](https://api.travis-ci.org/Juici/mvr.svg?branch=master)](https://travis-ci.org/Juici/mvr)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![crates.io](https://img.shields.io/crates/v/mvr.svg)](https://crates.io/crates/mvr)

**A utility program for renaming files.**

This program is a utility for renaming files using regex pattern matching.


### Features

- Regex pattern matching and replacement strings
- Bulk file renaming
- Perform a dry run before renaming
- Prompt on file renaming


## Install

From crates.io:
```
$ cargo install mvr
```

Manually:
```
$ git clone https://github.com/Juici/mvr
$ cd mvr
$ cargo install --path=./
```


## Build

```
$ git clone https://github.com/Juici/mvr
$ cd mvr
$ cargo build --release
```