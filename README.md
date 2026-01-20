# fixsha

Hook to automatically build a Nix derivation, detect if `cargoHash` was changed and update `package.nix` accordingly.

I know this exists somewhere but it took me less time to build a solution for my needs than to 
use the convuluted mess that's offered elsewhere. 

## Install

```bash
cargo install fixsha
```

## Usage

This assumes that your projects root has a `default.nix` and `package.nix`:

```bash
fixsha
```

## Git Hook

To automatically fix hashes before committing, paste this into `.git/hooks/pre-commit`:

```bash
#!/bin/sh
fixsha
if [ -f package.nix ]; then
    git add package.nix
fi
```

Make it executable:

```bash
chmod +x .git/hooks/pre-commit
```

Now `fixsha` runs on every commit and stages any hash updates automatically.

# License
MIT OR Apache-2.0, your choice.
