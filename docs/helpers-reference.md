# Helpers Reference

This document provides a comprehensive reference for the helper functions available in `pkl-lib/helpers.pkl`.

## Key Constants

The helpers module provides constants for common key codes to improve readability:

### Letters
```pkl
const a = "a"
const b = "b"
// ... through ...
const z = "z"
```

### Numbers
```pkl
const num0 = "0"
const num1 = "1"
// ... through ...
const num9 = "9"
```

### Special Keys
```pkl
const escape = "escape"
const return_or_enter = "return_or_enter"
const tab = "tab"
const spacebar = "spacebar"
const hyphen = "hyphen"
const equal_sign = "equal_sign"
const open_bracket = "open_bracket"
const close_bracket = "close_bracket"
const backslash = "backslash"
const semicolon = "semicolon"
const quote = "quote"
const grave_accent_and_tilde = "grave_accent_and_tilde"
const comma = "comma"
const period = "period"
const slash = "slash"
```

### Navigation Keys
```pkl
const up_arrow = "up_arrow"
const down_arrow = "down_arrow"
const left_arrow = "left_arrow"
const right_arrow = "right_arrow"
const page_up = "page_up"
const page_down = "page_down"
const home = "home"
const end = "end"
const delete_or_backspace = "delete_or_backspace"
const delete_forward = "delete_forward"
```

### Function Keys
```pkl
const f1 = "f1"
const f2 = "f2"
// ... through ...
const f12 = "f12"
```

## Basic Helper Functions

### remap
Simple one-to-one key remapping.

```pkl
function remap(fromKey: String, toKey: String) -> SimpleModification
```

**Example:**
```pkl
remap("caps_lock", "escape")
```

### basic
Creates a basic manipulator with optional modifiers.

```pkl
function basic(
  fromKey: String,
  toKey: String,
  modifiers: List<String>? = null
) -> Manipulator
```

**Example:**
```pkl
basic("h", "left_arrow", List("left_control"))
```

## Caps Lock Functions

### capsLockToEscape
Maps Caps Lock to Escape (simple version).

```pkl
function capsLockToEscape() -> Rule
```

### capsLockToEscapeControl
Maps Caps Lock to Control when held, Escape when tapped.

```pkl
function capsLockToEscapeControl() -> Rule
```

### capsLockToModifier
Maps Caps Lock to any modifier when held, Escape when tapped.

```pkl
function capsLockToModifier(modifier: String? = "right_control") -> Rule
```

**Example:**
```pkl
capsLockToModifier("left_command")  // Caps Lock → Cmd when held, Esc when tapped
```

## Layer Functions

### layer
Creates a modifier-based layer (hold modifier + key).

```pkl
function layer(
  trigger: String,
  mappings: Mapping<String, String | ToEvent>,
  threshold: Int? = 200
) -> Rule
```

**Parameters:**
- `trigger`: The modifier key to activate the layer
- `mappings`: Key-to-action mappings for the layer
- `threshold`: Simultaneous threshold in milliseconds

**Example:**
```pkl
layer("left_control", new Mapping {
  ["h"] = "left_arrow"
  ["j"] = "down_arrow"
  ["k"] = "up_arrow"
  ["l"] = "right_arrow"
})
```

### simlayer
Creates a simultaneous key layer (press keys together).

```pkl
function simlayer(
  trigger: String,
  mappings: Mapping<String, String | ToEvent>,
  threshold: Int? = 200
) -> Rule
```

**Example:**
```pkl
simlayer("d", new Mapping {
  ["h"] = "left_arrow"
  ["j"] = "down_arrow"
  ["k"] = "up_arrow" 
  ["l"] = "right_arrow"
})
// Press d+h together → left arrow
```

### spaceMode
Special simlayer using spacebar as trigger.

```pkl
function spaceMode(
  mappings: Mapping<String, String | ToEvent>,
  threshold: Int? = 200
) -> Rule
```

**Features:**
- Spacebar still works normally when pressed alone
- Activates mappings when pressed with other keys

**Example:**
```pkl
spaceMode(new Mapping {
  ["h"] = "left_arrow"
  ["j"] = "down_arrow"
  ["k"] = "up_arrow"
  ["l"] = "right_arrow"
  ["u"] = new ToEvent { shell_command = "open -a 'Google Chrome'" }
})
```

### shiftLayer
Makes any key act as shift for letters and numbers.

```pkl
function shiftLayer(trigger: String) -> Rule
```

**Example:**
```pkl
shiftLayer("semicolon")  // Hold semicolon to type capitals
```

## Navigation Functions

### vimNavigation
Creates vim-style navigation bindings.

```pkl
function vimNavigation(modifierKey: String? = "left_control") -> Rule
```

**Default Mappings:**
- h/j/k/l → arrow keys
- 0 → home, 4 → end  
- g → page up, G → page down
- d → delete forward, x → backspace

### vimHomeRowNavigation
Returns just the h/j/k/l navigation mappings.

```pkl
function vimHomeRowNavigation() -> Mapping<String, String>
```

## Application Functions

### appSwitcher
Creates app launcher bindings.

```pkl
function appSwitcher(
  trigger: String,
  apps: Mapping<String, String>
) -> Rule
```

**Example:**
```pkl
appSwitcher("right_command", new Mapping {
  ["c"] = "Google Chrome"
  ["s"] = "Slack"
  ["t"] = "Terminal"
  ["v"] = "Visual Studio Code"
})
```

### launchApp
Creates a single app launcher.

```pkl
function launchApp(
  key: String,
  app: String,
  modifiers: List<String>? = null
) -> Manipulator
```

**Example:**
```pkl
launchApp("t", "Terminal", List("right_command"))
```

## Other Functions

### hyperKey
Makes a key send all modifiers (Cmd+Ctrl+Opt+Shift).

```pkl
function hyperKey(trigger: String) -> Rule
```

**Example:**
```pkl
hyperKey("caps_lock")  // Caps Lock becomes Hyper key
```

## Utility Functions

### charRange
Generates a character range.

```pkl
function charRange(start: String, end: String) -> List<String>
```

**Example:**
```pkl
charRange("a", "f")  // ["a", "b", "c", "d", "e", "f"]
```

### numRange
Generates number strings.

```pkl
function numRange(start: Int, end: Int) -> List<String>
```

**Example:**
```pkl
numRange(1, 5)  // ["1", "2", "3", "4", "5"]
```

### qwertySequence
Gets contiguous keys from QWERTY layout.

```pkl
function qwertySequence(startKey: String, length: Int) -> List<String>
```

**Example:**
```pkl
qwertySequence("a", 4)  // ["a", "s", "d", "f"]
```

## Predefined Key Sequences

```pkl
const qwertyTopRow = ["q", "w", "e", "r", "t", "y", "u", "i", "o", "p", "[", "]"]
const qwertyHomeRow = ["a", "s", "d", "f", "g", "h", "j", "k", "l", ";", "'"]
const qwertyBottomRow = ["z", "x", "c", "v", "b", "n", "m", ",", ".", "/"]
```

## Complete Example

Here's a complete configuration using multiple helpers:

```pkl
import "modulepath:/karabiner_pkl/lib/karabiner.pkl"
import "modulepath:/karabiner_pkl/lib/helpers.pkl"

simpleConfig: karabiner.SimpleConfig = new {
  complex_modifications = new karabiner.ComplexModifications {
    rules = List(
      // Caps Lock → Control/Escape
      helpers.capsLockToEscapeControl(),
      
      // Vim navigation with Control
      helpers.vimNavigation(),
      
      // Space mode for quick actions
      helpers.spaceMode(new Mapping {
        ["h"] = helpers.left_arrow
        ["j"] = helpers.down_arrow
        ["k"] = helpers.up_arrow
        ["l"] = helpers.right_arrow
        ["u"] = new karabiner.ToEvent { 
          shell_command = "open -a 'Google Chrome'" 
        }
      }),
      
      // App switcher with right command
      helpers.appSwitcher("right_command", new Mapping {
        ["c"] = "Google Chrome"
        ["s"] = "Slack"
        ["t"] = "Terminal"
        ["v"] = "Visual Studio Code"
      }),
      
      // Semicolon as shift layer
      helpers.shiftLayer("semicolon")
    )
  }
}

config: karabiner.Config = simpleConfig.toConfig()
```

## Best Practices

1. **Use Constants**: Prefer `helpers.escape` over `"escape"` for better readability
2. **Compose Layers**: Combine multiple layer types for rich functionality
3. **Keep It Simple**: Start with basic helpers before creating custom manipulators
4. **Test Incrementally**: Add one rule at a time and test
5. **Document Complex Rules**: Add descriptions to your rules for clarity