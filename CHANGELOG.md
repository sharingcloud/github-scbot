# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Adding missing CA certificates in Docker image
- Removing unneeded fields from GitHub types

## [0.9.1] - 2021-03-01

### Changed

- Use a threadpool for a few database operations

## [0.9.0] - 2021-02-25

### Added

- React to GitHub webhooks to update review status
- Generate a summary comment, once per PR, automatically updated on lifecycle changes
- Validate PR titles depending on per-repository regexes (and a default configuration)
- Step system to track PR state (awaiting-checks, awaiting-changes, awaiting-merge, etc.)
- Merge support with merge rules depending on head and base branches (specific merge strategies)
- Automerge support when all steps are ok (awaiting-merge)
- React to issue comments: set/unset/skip QA status, ping bot, lock or unlock PR, merge PR, enable/disable automerge, post GIF
- Actions that can be triggered from external sources, with simple token-based authentication (JWT RS256), using registered external accounts (each account has a RSA key-pair)
- Give rights to external accounts on specific repositories
- Simple terminal UI interface to have an overview on pull requests

[Unreleased]: https://github.com/sharingcloud/github-scbot/compare/v0.9.1...HEAD
[0.9.1]: https://github.com/sharingcloud/github-scbot/compare/v0.9.0...v0.9.1
[0.9.0]: https://github.com/sharingcloud/github-scbot/compare/6d8ff170f7f36cc91a37e3af3766f62a3caefbe2...v0.9.0
