# Project structure

The project is split in (a lot of) multiple crates:

- `github-scbot-cli`:
    - It's the main entry-point, used to glue everything else through a command-line interface.
- `github-scbot-config`:
    - Expose the centralized configuration structure, obtained from environment variables; used by the other crates.
- `github-scbot-crypto`:
    - Expose cryptoraphic utilities (RSA keys generation, JWT creation and validation, etc.).
- `github-scbot-database-interface`:
    - Expose the database service interface (through the `DbService` trait), and a import/export system, without any implementation.
- `github-scbot-database-memory`:
    - Expose an in-memory database, implementing `DbService` (from `github-scbot-database-interface`).
- `github-scbot-database-pg`:
    - Expose a database service using Postgres, implementing `DbService` (from `github-scbot-database-interface`), with associated migrations.
- `github-scbot-database-tests`:
    - This is a crate used to run tests on available `DbService` implementations, to make sure they have the same results.
- `github-scbot-domain`:
    - The main crate of the project, which contains all the business logic (through multiple use cases and use cases interfaces).
- `github-scbot-domain-models`:
    - Expose the domain entities through all other crates.
- `github-scbot-ghapi-github`:
    - Expose a GitHub + Tenor API client, implementing `ApiService` (from `github-scbot-ghapi-interface`).
- `github-scbot-ghapi-interface`:
    - Expose the API service interface (through the `ApiService` trait), used to execute external API calls.
- `github-scbot-lock-interface`:
    - Expose the lock service interface (through the `LockService` trait), used to create application locks.
- `github-scbot-lock-redis`:
    - Expose a lock service using Redis, implementing `LockService` (from `github-scbot-lock-interface`).
- `github-scbot-logging`:
    - Expose a centralized logging configuration, based on `tracing`.
- `github-scbot-sentry`:
    - Expose utilities and integration around the *Sentry* platform.
- `github-scbot-server`:
    - Expose the HTTP server used to listen for the GitHub webhook requests, and external API calls.
- `github-scbot-tui`:
    - Expose a TUI application to browse through the known repositories, pull-requests, and their status.