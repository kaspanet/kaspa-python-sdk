# Releasing

## CI/CD Workflows

Three GitHub Actions workflows are in place:

### `ci.yml` — Continuous Integration

**Triggers:** All pushes and pull requests

| Job | Description |
|-----|-------------|
| **Lint** | `cargo fmt --check`, `cargo clippy` |
| **Build & Test** | Build package, run unit tests |
| **Docs** | Verify documentation builds |

### `docs.yml` — Documentation Deployment

**Triggers:** Push to `main`, version tags, manual dispatch

Builds versioned documentation with [MkDocs](https://www.mkdocs.org/) + [mike](https://github.com/jimporter/mike) and deploys to `gh-pages` branch.

| Trigger | Version on Docs Site | Alias |
|---------|---------|-------|
| Push to `main`/`poc` | `dev` | — |
| Tag `v1.0.0` | `1.0.0` | `latest` |
| Manual with input | (specified) | `latest` |

### `release.yml` — Release Builds

**Triggers:** GitHub Release published

Builds wheels for all platforms (Linux, macOS, Windows) and supported Python versions, then uploads to the GitHub Release.

## Branch & Tags

1. Development in feature branches, merged to `main`
2. Releases created by tagging commits with `vX.Y.Z`
3. Tags trigger automated builds and deployments

## Version Numbers

Versions are maintained in two files:

| File | Format | Example |
|------|--------|---------|
| `pyproject.toml` | PEP 440 | `2.0.0`, `2.0.0.dev0` |
| `Cargo.toml` | SemVer | `2.0.0`, `2.0.0-dev` |

**Progression:**

```
2.0.0.dev0 → 2.0.0a1 → 2.0.0b1 → 2.0.0rc1 → 2.0.0 → 2.1.0.dev0
```

## Changelog

[Keep a Changelog](https://keepachangelog.com/) format. Add to `[Unreleased]` during development with a target release version:

```md
## [Unreleased]

*Target: 1.1.0*
```

Then rename to `[X.Y.Z] - YYYY-MM-DD` at release.

## Release Checklist

### 1. Pre-Release Verification

```bash
./check  # Runs lints, tests, and doc build
```

### 2. Update Version & Changelog

Edit `pyproject.toml` and `Cargo.toml`:

```toml
version = "X.Y.Z"  # Remove dev suffix
```

Update `CHANGELOG.md`:

- Rename `[Unreleased]` → `[X.Y.Z] - YYYY-MM-DD`
- Add new empty `[Unreleased]` section

### 3. Commit, Tag, and Push to GitHub

### 4. Create GitHub Release

This triggers:

- `release.yml` → Builds and attaches wheels
- `docs.yml` → Deploys documentation

### 5. Upload built wheels to PyPi 

This step is currently manual but should be automated at some point.

### 5. Post-Release

Bump to next dev version:

```toml
# pyproject.toml
version = "X.Y.Z+1.dev0"

# Cargo.toml  
version = "X.Y.Z+1-dev"
```
