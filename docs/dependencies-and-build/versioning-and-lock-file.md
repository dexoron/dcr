# Versioning & lock file

DCR generates `dcr.lock` when dependency resolution succeeds and at least one dependency exists.

## What is stored

- Project entry (`name`, `version`, dependency names).
- One entry per resolved dependency:
  - `name`
  - `version` (from dependency `dcr.toml`, fallback `0.0.0`)
  - `source` (`path+...`)
  - `checksum` (SHA-256 over dependency files, excluding nested `target/`).

## Example shape

```toml
[[package]]
name = "my-app"
version = "0.1.0"
dependencies = ["fmt"]

[[package]]
name = "fmt"
version = "1.2.3"
source = "path+./third_party/fmt"
checksum = "..."
```

## Notes

- `dcr.lock` is rewritten on successful dependency resolution.
- If `[dependencies]` is empty, lock generation is skipped.
