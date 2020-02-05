# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog][], and this project adheres to
[Semantic Versioning][].

## Unreleased

## v0.1.0 – 2020-02-04

### Added

- `SOURCES` defaults to current working directory
- `<target>` defaults to home directory
- Recreate directory structure of `SOURCES` within `<target>`
- Link contents of `SOURCES` into corresponding locations within `<target>`
- Remove existing files in `<target>` if their type and size match the
  corresponding file in `SOURCES`
- Replace existing symbolic links in `<target>` with links to the corresponding
  file in `SOURCES`

[keep a changelog]: https://keepachangelog.com/en/1.0.0/
[semantic versioning]: https://semver.org/spec/v2.0.0.html
