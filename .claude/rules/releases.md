# Release Rules

## Release Process

1. Bump version in `Cargo.toml`
2. Commit: `chore: bump to vX.Y.Z`
3. Push commits to `origin/main`
4. Push the version tag: `git tag vX.Y.Z && git push origin vX.Y.Z`
5. The cargo-dist GitHub Actions workflow (`.github/workflows/release.yml`) handles everything else — it builds platform binaries and creates the GitHub Release automatically

## Do NOT

- **Never run `gh release create` manually.** The cargo-dist workflow creates the release. Manually creating it causes a "release with the same tag name already exists" error when the workflow runs.
- **Never create the tag before pushing commits.** Ensure all commits are on `origin/main` before pushing the tag.

## crates.io

- Run `cargo publish` separately after pushing the tag — cargo-dist does not publish to crates.io.
