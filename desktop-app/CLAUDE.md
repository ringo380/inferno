# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## ⚠️ Important Notice

**This directory (`desktop-app/`) is a minimal package placeholder for npm publishing.**

The actual Inferno Desktop application (Tauri v2 + Next.js) is located in:
```
../dashboard/
```

## Quick Navigation

If you need to work on the desktop application, navigate to the dashboard directory:

```bash
cd ../dashboard
```

## What's in This Directory?

This `desktop-app/` directory contains:
- Minimal `package.json` for npm package publishing to GitHub Packages
- Package metadata for `@ringo380/inferno-desktop` (v0.3.0)
- Basic npm scripts that delegate to Tauri commands

## Where to Work

| Task | Directory |
|------|-----------|
| Desktop app development | `../dashboard/` |
| Build desktop app | `../dashboard/` |
| Run desktop app | `../dashboard/` |
| Desktop UI components | `../dashboard/src/` |
| Tauri Rust backend | `../dashboard/src-tauri/src/` |

## Building the Desktop App

**Do NOT use the scripts in this directory.** Instead, use the main build script:

```bash
cd ..
./scripts/build-desktop.sh --release
```

Or work directly in the dashboard directory:

```bash
cd ../dashboard
npm run tauri:dev           # Development mode
npm run tauri:build         # Production build
npm run dev                 # Next.js dev server only
```

## See Also

- Main project documentation: `../CLAUDE.md`
- Desktop app README: `../dashboard/README.md`
- Build script: `../scripts/build-desktop.sh`

## This Directory's Purpose

This directory exists to:
1. Provide a publishable npm package for `@ringo380/inferno-desktop`
2. Maintain package metadata separate from the main dashboard codebase
3. Enable GitHub Packages distribution

For all development work, navigate to `../dashboard/`.