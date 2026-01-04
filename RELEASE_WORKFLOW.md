# Release Workflow Guide

## Overview

This workflow provides **hybrid auto-tagging** - you manually control versions, but the workflow automates everything else.

## How It Works

### On Every Push to `main`:

1. ✅ **Builds Docker image** with `latest` tag
2. ✅ **Pushes to GitHub Container Registry**

### When Version Changes in `Cargo.toml`:

1. ✅ **Creates Git tag** (e.g., `v0.5.0`)
2. ✅ **Creates GitHub Release** with changelog
3. ✅ **Builds both binaries** (sync + setup)
4. ✅ **Uploads binaries** to the release
5. ✅ **Tags Docker images** with version numbers

## Creating a New Release

### Step 1: Update Version

Edit `Cargo.toml` and bump the version:

```toml
[package]
name = "sparebank1-to-ynab"
version = "0.5.0"  # Changed from 0.4.0
edition = "2024"
```

### Step 2: Update Cargo.lock

```bash
cargo build
```

This updates `Cargo.lock` with the new version.

### Step 3: Commit and Push

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to 0.5.0"
git push origin main
```

### Step 4: Workflow Runs Automatically

The workflow will:
- Detect version change (`0.4.0` → `0.5.0`)
- Create tag `v0.5.0`
- Build binaries
- Create GitHub Release
- Upload binaries to release
- Push Docker images:
  - `ghcr.io/bjorngi/sparebank1-to-ynab/sparebank1-to-ynab-sync:latest`
  - `ghcr.io/bjorngi/sparebank1-to-ynab/sparebank1-to-ynab-sync:0.5.0`
  - `ghcr.io/bjorngi/sparebank1-to-ynab/sparebank1-to-ynab-sync:v0.5.0`

## Release Notes

The workflow automatically generates release notes from commits since the last tag:

```
## Changes

- Add structured logging with tracing (abc123)
- Add dry-run mode (def456)
- Add clap for improved CLI argument parsing (ghi789)
- Add comprehensive unit tests (jkl012)
```

## Versioning Guidelines

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (1.0.0): Breaking changes
  - API changes incompatible with previous versions
  - Configuration format changes
  - Example: `0.4.0` → `1.0.0`

- **MINOR** (0.5.0): New features (backwards compatible)
  - New functionality
  - New command-line options
  - Example: `0.4.0` → `0.5.0`

- **PATCH** (0.4.1): Bug fixes
  - Bug fixes
  - Performance improvements
  - Documentation updates
  - Example: `0.4.0` → `0.4.1`

## Example Release Workflow

### Feature Release (0.4.0 → 0.5.0)

You've added logging, dry-run, clap CLI, and tests on your branch `logs-and-such`:

```bash
# Merge your branch to main
git checkout main
git merge logs-and-such

# Bump version
# Edit Cargo.toml: version = "0.5.0"
cargo build

# Commit and push
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to 0.5.0 for feature release

Includes:
- Structured logging with tracing
- Dry-run mode
- Clap CLI improvements
- Comprehensive unit tests"

git push origin main
```

**Result:** Release `v0.5.0` created with all binaries and Docker images

### Bug Fix Release (0.5.0 → 0.5.1)

```bash
# Fix a bug
git commit -m "fix: prevent duplicate imports"

# Bump version
# Edit Cargo.toml: version = "0.5.1"
cargo build

# Commit and push
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to 0.5.1 for bug fix"
git push origin main
```

**Result:** Release `v0.5.1` created

### No Release (Development)

```bash
# Make changes without bumping version
git commit -m "docs: update README"
git push origin main
```

**Result:** Only Docker `latest` tag updated, no release created

## What Gets Created

### Git Tag
- Format: `vX.Y.Z`
- Example: `v0.5.0`
- Pushed to GitHub automatically

### GitHub Release
- **Name:** Release v0.5.0
- **Tag:** v0.5.0
- **Changelog:** Auto-generated from commits
- **Binaries:**
  - `sparebank1-to-ynab-sync-0.5.0-linux-x86_64`
  - `sparebank1-to-ynab-setup-0.5.0-linux-x86_64`

### Docker Images
- `ghcr.io/bjorngi/sparebank1-to-ynab/sparebank1-to-ynab-sync:latest`
- `ghcr.io/bjorngi/sparebank1-to-ynab/sparebank1-to-ynab-sync:0.5.0`
- `ghcr.io/bjorngi/sparebank1-to-ynab/sparebank1-to-ynab-sync:v0.5.0`

## Troubleshooting

### Release Not Created

**Check:**
1. Did you update `Cargo.toml` version?
2. Did you commit `Cargo.lock`?
3. Did the version actually change from previous commit?

**View workflow logs:**
- Go to Actions tab in GitHub
- Click on latest workflow run
- Check "Check if version changed" step

### Wrong Version in Docker Tag

The Docker tag uses the version from `Cargo.toml`. Ensure:
1. You updated the version correctly
2. You committed the change
3. The workflow ran after the commit

### Binaries Not Uploaded

Check that:
1. Version changed (releases only on version change)
2. Binaries built successfully
3. `permissions: contents: write` is set in workflow

## Manual Release (If Needed)

If the workflow fails or you need to create a release manually:

```bash
# Create tag
git tag -a v0.5.0 -m "Release v0.5.0"
git push origin v0.5.0

# Build binaries locally
cargo build --release --target x86_64-unknown-linux-gnu

# Create GitHub release manually through UI
# Upload binaries from target/x86_64-unknown-linux-gnu/release/
```

## Benefits of This Approach

✅ **You control versioning** - Explicit version updates in Cargo.toml
✅ **Automatic tagging** - No need to create Git tags manually
✅ **Automatic releases** - GitHub releases with binaries
✅ **Automatic Docker tags** - Multiple tags per version
✅ **Simple** - No complex dependencies or tools
✅ **Transparent** - Easy to understand what's happening
✅ **Flexible** - Push without releasing (don't bump version)

## Comparison with Full Semantic Release

| Feature | Hybrid (This) | Full Semantic |
|---------|---------------|---------------|
| Version control | Manual | Automatic |
| Commit format | Any | Conventional |
| Complexity | Low | Medium |
| Flexibility | High | Medium |
| Learning curve | None | Some |
| Dependencies | None | Node.js |

This hybrid approach gives you the best of both worlds: full control with automation where it matters.