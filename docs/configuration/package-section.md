# Package section

## Schema

```toml
[package]
name = "my-app"
version = "0.1.0"
```

## Fields

- `name` (string, required): used as output artifact base name.
- `version` (string, required): used in `dcr.lock` project entry.

## Notes

- Empty `name` or `version` makes config invalid.
- `dcr new` sets `name` from the passed project name.
- `dcr init` sets `name` from current directory name.
