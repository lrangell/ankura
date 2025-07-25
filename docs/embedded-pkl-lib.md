# Embedded Pkl Library

Starting with version 0.2.0, karabiner-pkl embeds the pkl library files directly in the binary. This means you no longer need to manually install or manage the pkl-lib files.

## How It Works

1. The pkl-lib files (`karabiner.pkl` and `helpers.pkl`) are embedded in the binary using rust-embed
2. At runtime, these files are materialized to:
   - macOS: `~/Library/Application Support/karabiner-pkl/`
   - Linux: `~/.local/share/karabiner-pkl/`
3. The pkl compiler is configured to find them via the `--module-path` option
4. Files are only extracted when missing or when the embedded version changes

## Import Syntax

Use the `modulepath:` URI scheme to import the embedded library:

```pkl
module karabiner_config

import "modulepath:/karabiner.pkl" as karabiner
import "modulepath:/helpers.pkl" as helpers

// Your configuration here
```

## Migration from Relative Imports

If you have existing configurations using relative imports like:
```pkl
import "karabiner_pkl/lib/karabiner.pkl"
```

You need to update them to use the `modulepath:` scheme:
```pkl
import "modulepath:/karabiner.pkl"
```

## Technical Details

- Files are embedded at compile time from the `pkl-lib/` directory
- Materialized to platform-specific data directory for persistent access
- A hash file (`.pkl-lib-hash`) tracks the embedded version to detect updates
- Module resolution works for both embedded files and user-provided modules in `~/.config/karabiner_pkl/lib/`

## LSP Support

For LSP support, the `init` command generates a `PklProject` file that configures the module path:

```pkl
amends "pkl:Project"

evaluatorSettings {
  modulePath {
    "~/.local/share/karabiner-pkl"
  }
}
```

This allows your editor's Pkl LSP to find and provide autocomplete for the karabiner-pkl library files.