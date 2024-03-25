# Project structure

The project is split in (a lot of) multiple crates:

- `prbot`:
    - It's the main entry-point, used to glue everything else through a command-line interface.
- `prbot-config`:
    - Expose the centralized configuration structure, obtained from environment variables; used by the other crates.
- `prbot-core`:
    - The main crate of the project, which contains all the business logic (through multiple use cases and use cases interfaces).
- `prbot-crypto`:
    - Expose cryptoraphic utilities (RSA keys generation, JWT creation and validation, etc.).
- `prbot-database-interface`:
    - Expose the database service interface (through the `DbService` trait), and a import/export system, without any implementation.
- `prbot-database-memory`:
    - Expose an in-memory database, implementing `DbService` (from `prbot-database-interface`).
- `prbot-database-pg`:
    - Expose a database service using Postgres, implementing `DbService` (from `prbot-database-interface`), with associated migrations.
- `prbot-database-tests`:
    - This is a crate used to run tests on available `DbService` implementations, to make sure they have the same results.
- `prbot-ghapi-github`:
    - Expose a GitHub + Tenor API client, implementing `ApiService` (from `prbot-ghapi-interface`).
- `prbot-ghapi-interface`:
    - Expose the API service interface (through the `ApiService` trait), used to execute external API calls.
- `prbot-lock-interface`:
    - Expose the lock service interface (through the `LockService` trait), used to create application locks.
- `prbot-lock-redis`:
    - Expose a lock service using Redis, implementing `LockService` (from `prbot-lock-interface`).
- `prbot-logging`:
    - Expose a centralized logging configuration, based on `tracing`.
- `prbot-models`:
    - Expose the domain entities through all other crates.
- `prbot-sentry`:
    - Expose utilities and integration around the *Sentry* platform.
- `prbot-server`:
    - Expose the HTTP server used to listen for the GitHub webhook requests, and external API calls.
- `prbot-tui`:
    - Expose a TUI application to browse through the known repositories, pull-requests, and their status.
