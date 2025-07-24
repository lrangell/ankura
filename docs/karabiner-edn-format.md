# Karabiner EDN Format

## Overview

The EDN (Extensible Data Notation) format is used by Goku (a Karabiner configuration tool) to provide a more concise and readable way to configure Karabiner-Elements. This document explains the EDN format structure and how it maps to Karabiner's JSON format.

## EDN vs JSON

EDN is a Clojure-inspired data format that offers several advantages over JSON for configuration:
- More concise syntax
- Comments support
- Keywords (`:keyword`) for cleaner structure
- Symbolic shortcuts for modifiers

## Basic Structure

```clojure
{:profiles {:Default {:default true
                      :sim 200     ; simultaneous_threshold_milliseconds
                      :delay 580   ; to_delayed_action_delay_milliseconds
                      :alone 1000  ; to_if_alone_timeout_milliseconds
                      :held 1000}} ; to_if_held_down_threshold_milliseconds
 :templates {...}
 :applications {...}
 :simlayers {...}
 :tos {...}
 :main [...]}
```

## Profiles Section

Maps to Karabiner's profile configuration:

```clojure
:profiles {:Default {:default true
                     :sim 200     ; simultaneous_threshold_milliseconds
                     :delay 580   ; to_delayed_action_delay_milliseconds
                     :alone 1000  ; to_if_alone_timeout_milliseconds
                     :held 1000}} ; to_if_held_down_threshold_milliseconds
```

Converts to:

```json
{
  "profiles": [{
    "name": "Default",
    "selected": true,
    "complex_modifications": {
      "parameters": {
        "basic.simultaneous_threshold_milliseconds": 200,
        "basic.to_delayed_action_delay_milliseconds": 580,
        "basic.to_if_alone_timeout_milliseconds": 1000,
        "basic.to_if_held_down_threshold_milliseconds": 1000
      }
    }
  }]
}
```

## Templates Section

Defines reusable command templates:

```clojure
:templates {:open "open \"%s\""
            :y-desk "/opt/homebrew/bin/yabai -m space --focus %s"
            :jump "osascript -e 'tell application \"System Events\" to keystroke \"%s\" as text'"}
```

Templates can be referenced in rules using their keyword names.

## Applications Section

Maps application bundle identifiers to short names:

```clojure
:applications {:kitty ["net.kovidgoyal.kitty"]
               :safari ["^com.apple.Safari$"]}
```

Used in conditional rules for app-specific mappings.

## Simlayers Section

Defines simultaneous key layers (hold one key + press another):

```clojure
:simlayers {:yabai-mode {:key :d}
            :f-mode {:key :f}
            :semicolon-as-shift {:key :semicolon}}
```

Each simlayer creates a modifier-like behavior for the specified key.

## TOS (To Shortcuts) Section

Defines reusable key combinations and named symbols:

```clojure
:tos {:exclaim {:key :1 :modi :shift}           ; !
      :at {:key :2 :modi :shift}                ; @
      :kitty-prev_tab {:key :!Oleft_arrow}      ; Option+Left
      :kitty-next_tab {:key :!Oright_arrow}}     ; Option+Right
```

## Main Rules Section

Contains the actual key mapping rules:

```clojure
:main [{:des "Description of rule"
        :rules [[:from_key :to_key conditions {:alone :key}]]}]
```

### Rule Format

Each rule in `:rules` follows this pattern:
```clojure
[:from_key :to_key conditions options]
```

- **from_key**: The key to capture (with optional modifiers)
- **to_key**: The key/action to trigger
- **conditions**: Optional conditions for the rule
- **options**: Optional behaviors like `:alone`, `:held`, `:delayed`

### Modifier Shortcuts

EDN uses single-letter shortcuts for modifiers:

| Symbol | Meaning | Karabiner Modifier |
|--------|---------|-------------------|
| `!` | Mandatory | Required modifier |
| `#` | Optional | Optional modifier |
| `C` | Command | left_command |
| `T` | Control | left_control |
| `O` | Option | left_option |
| `S` | Shift | left_shift |
| `F` | Function | fn |
| `Q` | Right Command | right_command |
| `W` | Right Control | right_control |
| `E` | Right Option | right_option |
| `R` | Right Shift | right_shift |
| `P` | Caps Lock | caps_lock |
| `!!` | Hyper | Cmd+Ctrl+Opt+Shift |
| `##` | Any | optional any |

### Examples

```clojure
; Caps Lock to Escape (alone) or Control (held)
[:##caps_lock :right_control nil {:alone :escape}]

; Command+H to Left Arrow
[:!Ch :left_arrow]

; D+J in simlayer to focus desktop 2
[:j [:y-desk 2]]  ; when inside :yabai-mode simlayer
```

## Conversion Process

The EDN format is converted to Karabiner JSON through these steps:

1. **Profile Expansion**: Profile parameters are expanded to full JSON paths
2. **Template Resolution**: Template strings are interpolated with parameters
3. **Simlayer Generation**: Each simlayer creates multiple simultaneous key rules
4. **Modifier Translation**: Single-letter modifiers are expanded to full names
5. **Rule Compilation**: Each EDN rule generates a complete manipulator object

### Simlayer Example

EDN:
```clojure
:simlayers {:f-mode {:key :f}}
:main [{:rules [:f-mode
                [:h :left_arrow]
                [:j :down_arrow]]}]
```

Generates JSON:
```json
{
  "manipulators": [{
    "type": "basic",
    "from": {
      "simultaneous": [
        {"key_code": "f"},
        {"key_code": "h"}
      ],
      "simultaneous_options": {
        "key_down_order": "strict",
        "key_up_order": "strict_inverse",
        "to_after_key_up": [
          {"set_variable": {"name": "f-mode", "value": 0}}
        ]
      }
    },
    "to": [{"key_code": "left_arrow"}]
  }]
}
```

## Key Advantages

1. **Conciseness**: Less verbose than JSON
2. **Readability**: Cleaner syntax with meaningful shortcuts
3. **Reusability**: Templates and named key combinations
4. **Comments**: Inline documentation support
5. **Layers**: Easy simlayer definition

## Tools

- **Goku**: The primary tool for converting EDN to Karabiner JSON
- **karabiner.edn**: User configuration file (typically in `~/.config/`)
- **karabiner.json**: Generated output file in `~/.config/karabiner/`

The EDN format significantly simplifies complex Karabiner configurations while maintaining full compatibility with all Karabiner features.