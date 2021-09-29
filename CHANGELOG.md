# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- New `strategy_override` field on the `PullRequest` table
- New `set-merge-strategy` command on `pull-requests` to set an overriden merge strategy for a specific pull request
- New `strategy+ <strategy>` and `strategy-` bot commands
- Handle commands from the pull request body at creation

### Changed

- Replaced `structopt` with `argh`
- Split the `github_scbot_libs` crate
- Better testability with `AppContext`
- Current status text is now in summary comment

## [0.12.0] - 2021-09-21

### Added

- Add default automerge, QA, and checks status in repository
- Add `nochecks+` and `nochecks-` bot commands
- Add `admin-set-default-automerge+` and `admin-set-default-automerge-` bot commands
- Add `admin-set-default-qa-status+` and `admin-set-default-qa-status-` bot commands
- Add `admin-set-default-checks-status+` and `admin-set-default-checks-status-` bot commands
- Add `repository set-automerge <status>` CLI command
- Add `repository set-qa-status <status>` CLI command
- Add `repository set-checks-status <status>` CLI command

## [0.11.0] - 2021-09-13

### Added

- `admin-reset-reviews` command to reset stored reviews
- Add optional merge strategy override to `bot merge` command

### Fixed

- Remove reviews before removing pull request (especially needed on `bot admin-disable`)
- Handle empty body on issue/pull request

### Changed

- Split Command types in two User / Admin enums

## [0.10.1] - 2021-06-14

### Fixed

- GitHub token regenerated at each request (if not, bad credentials)

## [0.10.0] - 2021-06-13

### Added

- Configurable database pool size (`BOT_DATABASE_POOL_SIZE`)
- Redis support in crate `github_scbot_redis`, mostly to set locks
- Using adapters on each external part: API, database, Redis, gifs
- `admin-disable` command to disable bot on a PR (only in manual interaction mode)

### Changed

- All database calls are now asynchronous, using a separate threadpool (using `tokio_diesel`)
- Use Rust `nightly-2021-06-04` for bleeding edge `rustfmt`, `clippy` and `grcov` compatibility
- Renamed the `github_scbot` crate to `github_scbot_cli`

### Fixed

- All admin commands are now checking admin rights
- Summary message is now only created on PR opening, or after `admin-enable` command
- Thanks to Redis locks, there should be no more race conditions on automerge

## [0.9.3] - 2021-05-19

### Added

- Error handling on bot commands

### Fixed

- Ignore check suites without pull requests

## [0.9.2] - 2021-05-17

### Added

- Added a `debug test-sentry` command to test Sentry connection
- Manual interaction mode (using admin-enable command) to use the bot on specific pull requests only
- Optional history tracking
- Support for more admin commands (`set-default-needed-reviewers`, `set-default-merge-strategy`, `set-default-pr-title-regex`, `set-needed-reviewers`)

### Fixed

- Removed unneeded fields from GitHub types
- Fixed status duplications (only PR opening should trigger PR creation in database)

### Changed

- Use Rust nightly for formatting (more options)
- Use buster-slim base Docker image
- Renaming admin commands with `admin-` suffix (admin-help, admin-enable, admin-sync)
- Merge command results in one comment (if comments are sent)
- Better Gif parameters

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

[Unreleased]: https://github.com/sharingcloud/github-scbot/compare/v0.11.0...HEAD
[0.12.0]: https://github.com/sharingcloud/github-scbot/compare/v0.11.0...v0.12.0
[0.11.0]: https://github.com/sharingcloud/github-scbot/compare/v0.10.1...v0.11.0
[0.10.1]: https://github.com/sharingcloud/github-scbot/compare/v0.10.0...v0.10.1
[0.10.0]: https://github.com/sharingcloud/github-scbot/compare/v0.9.3...v0.10.0
[0.9.3]: https://github.com/sharingcloud/github-scbot/compare/v0.9.2...v0.9.3
[0.9.2]: https://github.com/sharingcloud/github-scbot/compare/v0.9.1...v0.9.2
[0.9.1]: https://github.com/sharingcloud/github-scbot/compare/v0.9.0...v0.9.1
[0.9.0]: https://github.com/sharingcloud/github-scbot/compare/6d8ff170f7f36cc91a37e3af3766f62a3caefbe2...v0.9.0
