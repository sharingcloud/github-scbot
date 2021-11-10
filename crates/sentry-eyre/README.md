<p align="center">
    <a href="https://sentry.io" target="_blank" align="center">
        <img src="https://sentry-brand.storage.googleapis.com/sentry-logo-black.png" width="280">
    </a>
</p>

# sentry-eyre

Adds support for capturing Sentry errors from `eyre::Report`.

## Example

```rust
use sentry_eyre::capture_eyre;

fn function_that_might_fail() -> eyre::Result<()> {
    Err(eyre::eyre!("some kind of error"))
}

if let Err(err) = function_that_might_fail() {
    capture_eyre(&err);
}
```

## Resources

License: Apache-2.0
