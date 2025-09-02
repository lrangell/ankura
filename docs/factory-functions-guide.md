# Factory Functions Guide

This guide covers the new factory functions API introduced in Karabiner-Pkl, which provides a clean, readable alternative to complex nested object construction.

## Overview

The factory functions API eliminates the need for verbose `new` constructions while maintaining full type safety and backward compatibility.

### Before vs After

**Before (Old API):**
```pkl
new karabiner.SimpleConfig {
  complex_modifications = new karabiner.ComplexModifications {
    rules = List(
      new karabiner.Rule {
        description = "Caps Lock to Escape when alone, Control when held"
        manipulators = List(
          new karabiner.Manipulator {
            type = "basic"
            from = new karabiner.FromEvent { key_code = "caps_lock" }
            to = List(new karabiner.ToEvent { key_code = "left_control" })
            to_if_alone = List(new karabiner.ToEvent { key_code = "escape" })
          }
        )
      }
    )
  }
}.toConfig()
```

**After (New API):**
```pkl
k.config("Profile", List(
  k.rule("Caps dual-use", k.dualUse(keys.caps_lock, keys.left_control, keys.escape))
))
```

## Core Principles

### 1. Consistent Argument Order
All functions follow the pattern: `<modifier>` (if applicable), `<from>`, `<to>`

```pkl
k.map(keys.a, keys.b)                           // <from>, <to>
k.withMods(keys.cmd, keys.t, keys.tab)          // <modifier>, <from>, <to>
k.dualUse(keys.space, keys.opt, keys.space)     // <key>, <hold>, <tap>
```

### 2. Flexible Input Types
Functions accept both strings and complex objects:

```pkl
// String input
k.map("a", "b")

// Key constants  
k.map(keys.a, keys.b)

// Complex ToEvent objects
k.map(keys.t, h.shell("open -a 'Terminal'"))

// Lists of actions
k.map(keys.a, List(keys.b, keys.c))
```

### 3. Single Source of Truth
All key constants come from the dedicated `keys.pkl` module:

```pkl
import "modulepath:/keys.pkl"

// Use validated key constants
keys.caps_lock    // "caps_lock"
keys.escape       // "escape"  
keys.left         // "left_arrow"
```

## Factory Function Reference

### Basic Mappings

#### `map(from, to)`
Simple key remapping.

```pkl
// Basic remapping
k.map(keys.caps_lock, keys.escape)

// Multiple key output
k.map(keys.a, List(keys.b, keys.c))
```

#### `withMods(modifiers, from, to)`
Key mapping with modifier requirements.

```pkl
// Single modifier
k.withMods(keys.cmd, keys.t, keys.tab)

// Multiple modifiers  
k.withMods(List(keys.cmd, keys.shift), keys.t, h.shell("open -a 'Terminal'"))
```

### Advanced Behaviors

#### `dualUse(key, hold, tap)`
Different behavior when key is held vs tapped.

```pkl
// Caps lock: Control when held, Escape when tapped
k.dualUse(keys.caps_lock, keys.left_control, keys.escape)

// Spacebar: Option when held, Space when tapped
k.dualUse(keys.spacebar, keys.left_option, keys.spacebar)
```

#### `simul(keys, to)`
Simultaneous key combinations.

```pkl
// String input - splits into individual keys
k.simul("jk", keys.escape)    // j+k simultaneously -> escape

// List input - explicit key list
k.simul(List("j", "k"), keys.escape)
```

### Layers

#### `layer(modifier, mappings)`
Modifier-based layers.

```pkl
k.layer(keys.left_option, k.mapping(new Mapping {
  ["h"] = keys.left_arrow
  ["j"] = keys.down_arrow
  ["k"] = keys.up_arrow  
  ["l"] = keys.right_arrow
}))
```

#### `simlayer(trigger, mappings)`
Simultaneous key layers.

```pkl
k.simlayer("f", k.mapping(new Mapping {
  ["h"] = keys.left_arrow
  ["j"] = keys.down_arrow
  ["k"] = keys.up_arrow
  ["l"] = keys.right_arrow
}))
```

### Structure Creation

#### `rule(description, manipulator)`
Create a rule with a single manipulator.

```pkl
k.rule("Caps to Escape", k.map(keys.caps_lock, keys.escape))
```

#### `config(profileName, rules)`
Create a complete configuration.

```pkl
k.config("MyProfile", List(
  k.rule("Rule 1", manipulator1),
  k.rule("Rule 2", manipulator2)
))
```

#### `mapping(entries)`
Create a mapping object (for layers).

```pkl
k.mapping(new Mapping {
  ["a"] = keys.left_arrow
  ["s"] = keys.down_arrow
})
```

## Helper Function Integration

Helper functions now use factory functions internally:

```pkl
import "modulepath:/helpers.pkl" as h

// Modern shortcuts using factory functions
h.capsToEsc()              // Caps lock -> Escape
h.capsToCtrlEsc()          // Caps lock dual-use  
h.spaceToOpt()             // Spacebar dual-use
h.vimNavigation()          // Vim navigation layer
h.shell("command")         // Shell command execution
```

## Complete Example

Here's a full configuration showcasing the new API:

```pkl
import "modulepath:/karabiner.pkl" as k
import "modulepath:/helpers.pkl" as h
import "modulepath:/keys.pkl"

config = k.config("DeveloperProfile", List(
  // Basic remappings
  k.rule("Caps to Escape", k.map(keys.caps_lock, keys.escape)),
  
  // Dual-use keys
  k.rule("Space dual-use", k.dualUse(keys.spacebar, keys.left_option, keys.spacebar)),
  
  // Simultaneous keys for quick access
  k.rule("JK to escape", k.simul("jk", keys.escape)),
  k.rule("FD to enter", k.simul("fd", keys.return_or_enter)),
  
  // Navigation layer
  k.layer(keys.left_option, k.mapping(new Mapping {
    ["h"] = keys.left_arrow
    ["j"] = keys.down_arrow
    ["k"] = keys.up_arrow
    ["l"] = keys.right_arrow
    ["semicolon"] = keys.return_or_enter
    ["i"] = keys.delete_or_backspace
  })),
  
  // App launchers
  k.rule("Terminal launcher", 
    k.withMods(List(keys.cmd, keys.shift), keys.t, h.shell("open -a 'Terminal'"))),
  k.rule("Browser launcher",
    k.withMods(List(keys.cmd, keys.shift), keys.b, h.shell("open -a 'Safari'"))),
  
  // Helper shortcuts
  h.vimNavigation(),
  h.capsToCtrlEsc()
))
```

## Advanced Usage

You can still create complex manipulators when needed by using the raw Karabiner types within the factory function framework:

```pkl
k.config("Advanced", List(
  // Use factory functions for common cases
  k.rule("Simple mapping", k.map(keys.a, keys.b)),
  
  // Use raw manipulators for complex scenarios
  k.rule("Complex behavior", new k.Manipulator {
    type = "basic"
    from = new k.FromEvent { key_code = keys.c }
    to = List(new k.ToEvent { key_code = keys.d })
    conditions = List(new k.Condition {
      type = "frontmost_application_if"
      bundle_identifiers = List("com.apple.Terminal")
    })
  })
))
```

This provides the best of both worlds: simple syntax for common cases and full power when needed.