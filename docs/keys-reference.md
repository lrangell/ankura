# Keys Reference

This document provides a comprehensive reference for the key constants available in `pkl/keys.pkl`.

## Overview

The keys module provides a single source of truth for all key constants used in Karabiner-Pkl configurations. This eliminates duplication, provides better organization, and includes helpful aliases.

```pkl
import "modulepath:/keys.pkl"

// Use key constants in your configuration
config = k.config("MyProfile", List(
  k.rule("Caps to Escape", k.map(keys.caps_lock, keys.escape))
))
```

## Key Categories

### Letters (a-z)
```pkl
keys.a = "a"
keys.b = "b"  
keys.c = "c"
// ... through ...
keys.z = "z"
```

### Numbers (0-9)
```pkl
// Full names
keys.zero = "0"
keys.one = "1"
keys.two = "2"
// ... through ...
keys.nine = "9"

// Aliases  
keys.num0 = "0"  // same as keys.zero
keys.num1 = "1"  // same as keys.one
// ... etc ...
```

### Special Keys
```pkl
keys.spacebar = "spacebar"
keys.space = "spacebar"           // alias

keys.escape = "escape"
keys.esc = "escape"               // alias

keys.enter = "return_or_enter"
keys.return_or_enter = "return_or_enter"

keys.delete = "delete_or_backspace"      // uses backticks due to keyword
keys.delete_or_backspace = "delete_or_backspace"
keys.backspace = "delete_or_backspace"   // alias

keys.delete_forward = "delete_forward"

keys.tab = "tab"
```

### Modifiers
```pkl
// Full names
keys.left_command = "left_command"
keys.left_control = "left_control"  
keys.left_option = "left_option"
keys.left_shift = "left_shift"
keys.right_command = "right_command"
keys.right_control = "right_control"
keys.right_option = "right_option"
keys.right_shift = "right_shift"

// Common aliases
keys.cmd = "left_command"
keys.ctrl = "left_control"
keys.control = "left_control"
keys.opt = "left_option"  
keys.option = "left_option"
keys.shift = "left_shift"
keys.fn = "fn"
```

### Arrow Keys
```pkl
// Full names
keys.left_arrow = "left_arrow"
keys.right_arrow = "right_arrow"
keys.up_arrow = "up_arrow"
keys.down_arrow = "down_arrow"

// Short aliases
keys.left = "left_arrow"
keys.right = "right_arrow"
keys.up = "up_arrow"
keys.down = "down_arrow"
```

### Function Keys
```pkl
keys.f1 = "f1"
keys.f2 = "f2"
keys.f3 = "f3"
// ... through ...
keys.f12 = "f12"
```

### Punctuation
```pkl
keys.semicolon = "semicolon"
keys.comma = "comma"
keys.period = "period"
keys.slash = "slash"
keys.backslash = "backslash"
keys.quote = "quote"
keys.grave_accent_and_tilde = "grave_accent_and_tilde"
keys.grave = "grave_accent_and_tilde"    // alias
keys.hyphen = "hyphen"
keys.equal_sign = "equal_sign"
keys.equal = "equal_sign"               // alias
keys.open_bracket = "open_bracket"
keys.close_bracket = "close_bracket"
```

### Navigation Keys
```pkl
keys.page_up = "page_up"
keys.page_down = "page_down"
keys.home = "home"
keys.end = "end"
```

### Caps Lock
```pkl
keys.caps_lock = "caps_lock"
keys.caps = "caps_lock"                 // alias
```

### Media Keys
```pkl
keys.play_or_pause = "play_or_pause"
keys.play_pause = "play_or_pause"       // alias
keys.play = "play_or_pause"             // alias
keys.volume_increment = "volume_increment"
keys.volume_decrement = "volume_decrement"
keys.volume_up = "volume_increment"     // alias
keys.volume_down = "volume_decrement"   // alias
keys.mute = "mute"
```

## Key Validation

The keys module includes a validation function to prevent typos:

```pkl
keys.validateKeyCode(key: String): String
```

This function:
- Validates that the key code exists
- Provides helpful error messages for common mistakes
- Is used internally by all factory functions

## Usage Patterns

### Using Aliases for Readability
```pkl
// More readable
k.layer(keys.opt, k.mapping(new Mapping {
  ["h"] = keys.left
  ["j"] = keys.down  
  ["k"] = keys.up
  ["l"] = keys.right
}))

// vs. full names
k.layer(keys.left_option, k.mapping(new Mapping {
  ["h"] = keys.left_arrow
  ["j"] = keys.down_arrow
  ["k"] = keys.up_arrow  
  ["l"] = keys.right_arrow
}))
```

### Mixing Keys and Strings
```pkl
// Keys module constants
k.map(keys.caps_lock, keys.escape)

// Raw strings (still valid)
k.map("caps_lock", "escape")  

// Mixed (not recommended but works)
k.map(keys.caps_lock, "escape")
```

### IDE Support

Using key constants provides better IDE support:
- **Autocomplete** - Type `keys.` to see all available options
- **Type checking** - Catch typos at compile time  
- **Go to definition** - Navigate to key definitions
- **Refactoring** - Safely rename keys across your configuration