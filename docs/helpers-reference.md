# Helpers Reference

This document provides a comprehensive reference for the helper functions available in `pkl/helpers.pkl`.

## Key Constants

Key constants are now provided by the dedicated `keys.pkl` module:

```pkl
import "modulepath:/keys.pkl"

// Use keys from the dedicated module
keys.caps_lock  // "caps_lock"
keys.escape     // "escape"
keys.left_arrow // "left_arrow"
```

See the [Keys Reference](keys-reference.md) for complete documentation.

## Helper Functions

Helper functions provide convenient shortcuts for common keyboard customization patterns:

### Common Shortcuts

#### Caps Lock Remapping
```pkl
// Simple caps lock to escape
h.capsToEsc()

// Caps lock to control when held, escape when tapped
h.capsToCtrlEsc()

// Custom modifier and tap behavior
h.capsLockToModifier(keys.right_control, keys.escape)
```

#### Spacebar Dual-Use
```pkl
// Spacebar to command when held, space when tapped
h.spaceToCmd()

// Spacebar to option when held, space when tapped
h.spaceToOpt()

// Spacebar to control when held, space when tapped
h.spaceToCtrl()
```

#### Navigation Layers
```pkl
// Vim navigation with default control modifier
h.vimNavigation()

// Vim navigation with custom modifier
h.vimNavigation(keys.left_option)
```

### Advanced Functions

#### Application Launching
```pkl
// Simple app launch
h.launchApp("t", "Terminal")

// App launch with modifier keys
h.launchApp("t", "Terminal", List(keys.cmd, keys.shift))
```

#### Shell Command Helper
```pkl
// Execute shell command
h.shell("open -a 'Terminal'")
```

## Available Helper Functions

The helpers module provides a clean interface built on the factory functions. All functions are implemented using the new factory functions API internally.