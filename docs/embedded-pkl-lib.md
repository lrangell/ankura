# Embedded Pkl Library

Starting with version 0.2.0, karabiner-pkl embeds the pkl library files directly in the binary. This means you no longer need to manually install or manage the pkl-lib files.

## How It Works

1. The pkl-lib files (`karabiner.pkl` and `helpers.pkl`) are embedded in the binary using rust-embed
2. At runtime, these files are extracted to a temporary directory
3. The pkl compiler is configured to find them via the `--module-path` option

## Import Syntax

Use the `modulepath:` URI scheme to import the embedded library:

```pkl
module karabiner_config

import "modulepath:/karabiner_pkl/lib/karabiner.pkl" as karabiner
import "modulepath:/karabiner_pkl/lib/helpers.pkl" as helpers

// Your configuration here
```

## Migration from Relative Imports

If you have existing configurations using relative imports like:
```pkl
import "karabiner_pkl/lib/karabiner.pkl"
```

You need to update them to use the `modulepath:` scheme:
```pkl
import "modulepath:/karabiner_pkl/lib/karabiner.pkl"
```

## Technical Details

- Files are embedded at compile time from the `pkl-lib/` directory
- Extracted to a temporary directory that matches the expected module structure
- The temporary directory is cleaned up when the program exits
- Module resolution works for both embedded files and user-provided modules in `~/.config/karabiner_pkl/lib/`