# GitHub Builds and Releases

Clodhopper uses GitHub Actions to create downloadable packages that can be installed outside the source tree.

## Workflows

### Build Workflow

File: `.github/workflows/build.yml`

Runs on:

- pushes to `main`
- pull requests
- manual `workflow_dispatch`

Jobs:

- `validate`: runs `cargo fmt -- --check` and `cargo test --locked`.
- `package`: builds release binaries and uploads platform archives as workflow artifacts.

Artifacts:

- `clodhopper-linux-x86_64.tar.gz`
- `clodhopper-macos-aarch64.tar.gz`
- `clodhopper-windows-x86_64.zip`

Use this workflow when you want a build to download and test before cutting a release.

### Release Workflow

File: `.github/workflows/release.yml`

Runs on:

- pushed tags matching `v*`
- manual `workflow_dispatch` with a tag input

The workflow builds the same packages and attaches them to a GitHub Release.

## Package Layout

macOS/Linux:

```text
clodhopper-<platform>/
  bin/
    harness
  install.sh
  README.md
  docs/
```

Windows:

```text
clodhopper-windows-x86_64/
  bin/
    harness.exe
  install.ps1
  README.md
  docs/
```

## Install Behavior

The installer copies the binary from the extracted package into a user bin directory.

Defaults:

- macOS/Linux: `~/.local/bin/harness`
- Windows: `$HOME\.local\bin\harness.exe`

Custom install locations:

```bash
./install.sh --dir "$HOME/bin"
```

```powershell
.\install.ps1 -InstallDir "$HOME\bin"
```

The installer verifies the copied binary by running `harness --help`.

## Manual Release Process

1. Ensure the working tree is clean.
2. Run local validation:

   ```bash
   cargo fmt -- --check
   cargo test --locked
   cargo build --release --locked
   ```

3. Commit changes.
4. Tag the release:

   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

5. Wait for the `Release` workflow.
6. Download and test the release artifact on a clean path:

   ```bash
   tar -xzf clodhopper-macos-aarch64.tar.gz
   cd clodhopper-macos-aarch64
   ./install.sh --dir "$HOME/.local/bin"
   harness doctor
   ```

## Notes

- The build workflow artifacts are useful for testing unreleased commits.
- The release workflow assets are the stable install path for users.
- Provider CLIs are not bundled. `harness doctor` reports which provider tools are available locally.

