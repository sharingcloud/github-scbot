# GitHub SC Bot

Experimental GitHub Bot to manage our development workflows.

## Why ?

GitHub is great and contains many useful features, but sometimes you need more.
Today, you can use labels to have a quick view on the steps of your workflow,
but you have to put them by hand, or write scripts to manipulate them.

One GitHub-hosted possibility is to use GitHub Actions as a workflow orchestrator
(listening on pull request updates, added comments, or even check runs), but GitHub Actions _are_ checks,
so they need to boot at each GitHub event you want to support. They will also show on your Actions list,
so your actual actions (as tests or builds) are not easily visible anymore.

Another problem with GA is that you need to duplicate your workflows for each of your repositories, even if you can use packages
(as on GitHub Packages).

So here is another idea, why not use a centralized bot, which can react to webhook events to manage your workflows, can handle multiple repositories at once,
maintaining state in a database and using per-repository configuration files ? Using the workflow events and GitHub API you can have almost real-time interactions.

Some bots like this already exists, like the Bors bot for the Rust programming language, but this bot will be more specifically designed for our own use-cases.
Il will add some features:

- Use a command system through issue comments to drive the bot
- Specify a regular expression to be matched for pull request titles
- Specify manual QA status for a pull request (skipped, fail, pass, waiting)
- Specify required reviewers for a particular pull request, and lock the merge if these reviews do not pass
- Apply specific labels on pull requests depending on their current state
- Maintain an automatically updated status comment with relevant details
- And maintain a commit status to prevent or allow merge

It will also be drivable via a command-line interface (for one-time actions as synchronizing state, or
import/export), and if possible a terminal-like user interface (TUI, à la htop).

## Features

- [x] Manage PR lifecycle with local data stored in a PostgreSQL database
- [x] Reacts to GitHub Webhooks to update review status
- [x] Generate a summary comment (once per PR)
- [x] Validate PR titles depending on per-repository regexes
- [x] Reacts to comments: Set QA status (or skip), ping, lock/unlock, merge, etc.
- [x] Require mandatory reviewers
- [x] Enable auto-merge
- [ ] Simple permission system to control who can type which command
- [ ] Terminal-like interface to manage pull request status
- [ ] Actions that can be triggered from external sources, with simple token-based authentication

## Step labels

Process can be followed with labels, which are auto applied depending on the current pull request state, in this order:

- PR is WIP? **step/wip**
- Waiting for checks? **step/awaiting-checks**
- Checks failed? **step/awaiting-changes**
- Waiting for required reviews? **step/awaiting-required-review**
- Waiting for reviews? **step/awaiting-reviews**
- Waiting for QA? **step/awaiting-qa**
- QA failed? **step/awaiting-changes**
- PR is locked? **step/locked**
- All good? **step/awaiting-merge**

## Building

This project is written in the [Rust programming language](https://www.rust-lang.org/), so to build you have to [install the Rust tools](https://www.rust-lang.org/tools/install).  
To easily execute build tasks, this project use **just** (https://github.com/casey/just), a modern Makefile-inspired command runner built in Rust.  
To install, type `cargo install just`.

You can then type `just --list` to print available commands.

## Developing

You can quick mount a PostgreSQL instance via Docker with the following commands:

    docker volume create postgres-data
    docker run -d \
        --name postgres \
        -p 5432:5432 \
        -v postgres-data:/var/lib/postgresql/data \
        -e POSTGRES_USER=user \
        -e POSTGRES_PASSWORD=pass \
        -e POSTGRES_DB=bot \
        postgres:alpine \
        -c max_connections=200
