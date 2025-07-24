# Karabiner JSON Structure

This document describes the structure of the `karabiner.json` configuration file that Karabiner-Elements uses. This is the output format that karabiner-pkl generates.

## Overview

The Karabiner configuration file is a JSON document with a single root object containing a `profiles` array. Each profile represents a complete keyboard configuration that can be switched between.

## Top-Level Structure

```json
{
  "profiles": [
    {
      // Profile configuration
    }
  ]
}
```

## Profile Structure

Each profile in the `profiles` array contains:

```json
{
  "name": "Default",
  "selected": true,
  "virtual_hid_keyboard": {
    "keyboard_type_v2": "ansi"
  },
  "complex_modifications": {
    // Complex modification rules
  },
  "fn_function_keys": [],
  "simple_modifications": [],
  "devices": []
}
```

### Profile Fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Display name of the profile |
| `selected` | boolean | Whether this profile is currently active |
| `virtual_hid_keyboard` | object | Virtual keyboard settings |
| `complex_modifications` | object | Advanced key mappings and rules |
| `fn_function_keys` | array | Function key mappings |
| `simple_modifications` | array | Simple one-to-one key remappings |
| `devices` | array | Device-specific settings |

## Complex Modifications

The `complex_modifications` object contains parameters and rules:

```json
{
  "parameters": {
    "basic.simultaneous_threshold_milliseconds": 200,
    "basic.to_delayed_action_delay_milliseconds": 500,
    "basic.to_if_alone_timeout_milliseconds": 1000,
    "basic.to_if_held_down_threshold_milliseconds": 500
  },
  "rules": [
    // Array of rules
  ]
}
```

### Parameters

Common parameters that control timing behavior:

| Parameter | Default | Description |
|-----------|---------|-------------|
| `basic.simultaneous_threshold_milliseconds` | 50 | Time window for simultaneous key presses |
| `basic.to_delayed_action_delay_milliseconds` | 500 | Delay before delayed actions trigger |
| `basic.to_if_alone_timeout_milliseconds` | 1000 | Maximum time for "if alone" behavior |
| `basic.to_if_held_down_threshold_milliseconds` | 500 | Time before "held down" behavior activates |

## Rules

Each rule in the `rules` array has:

```json
{
  "description": "Human-readable rule description",
  "manipulators": [
    // Array of manipulators
  ]
}
```

## Manipulators

Manipulators define the actual key transformations:

```json
{
  "type": "basic",
  "from": {
    "key_code": "caps_lock",
    "modifiers": {
      "mandatory": [],
      "optional": ["any"]
    }
  },
  "to": [
    {
      "key_code": "left_control"
    }
  ],
  "to_if_alone": [
    {
      "key_code": "escape"
    }
  ],
  "to_if_held_down": [],
  "to_after_key_up": [],
  "conditions": []
}
```

### Manipulator Fields

| Field | Type | Description |
|-------|------|-------------|
| `type` | string | Always "basic" for standard manipulators |
| `from` | object | The input event to match |
| `to` | array | Events to generate when key is pressed |
| `to_if_alone` | array | Events when key is pressed and released without other keys |
| `to_if_held_down` | array | Events when key is held down |
| `to_after_key_up` | array | Events to generate after key is released |
| `conditions` | array | Conditions that must be met for the manipulator to activate |

## From Event

The `from` object specifies what input to match:

```json
{
  "key_code": "a",
  "modifiers": {
    "mandatory": ["left_shift", "left_command"],
    "optional": ["caps_lock"]
  },
  "simultaneous": [
    {
      "key_code": "j"
    }
  ],
  "simultaneous_options": {
    "key_down_order": "strict",
    "key_up_order": "strict_inverse"
  }
}
```

## To Event

Each event in the `to` arrays can be:

```json
{
  "key_code": "tab",
  "modifiers": ["left_command"],
  "repeat": true,
  "shell_command": "open -a Terminal"
}
```

### Event Types

- **Key event**: `key_code` with optional `modifiers`
- **Shell command**: `shell_command` to execute
- **Mouse event**: Mouse button or movement
- **Variable**: Set or unset Karabiner variables

## Simple Modifications

For basic one-to-one key remapping:

```json
"simple_modifications": [
  {
    "from": {
      "key_code": "caps_lock"
    },
    "to": [
      {
        "key_code": "escape"
      }
    ]
  }
]
```

## Devices

Device-specific configurations:

```json
"devices": [
  {
    "identifiers": {
      "vendor_id": 1452,
      "product_id": 832,
      "is_keyboard": true,
      "is_pointing_device": false
    },
    "ignore": false,
    "disable_built_in_keyboard_if_exists": false,
    "fn_function_keys": [],
    "simple_modifications": []
  }
]
```

## Common Patterns

### Dual-Function Keys

Keys that act differently when tapped vs held:

```json
{
  "type": "basic",
  "from": {
    "key_code": "caps_lock"
  },
  "to": [
    {
      "key_code": "left_control"
    }
  ],
  "to_if_alone": [
    {
      "key_code": "escape"
    }
  ]
}
```

### Simultaneous Keys (Simlayer)

Trigger actions with simultaneous key presses:

```json
{
  "from": {
    "simultaneous": [
      {
        "key_code": "j"
      },
      {
        "key_code": "k"
      }
    ],
    "simultaneous_options": {
      "detect_key_down_uninterruptedly": true,
      "key_down_order": "strict",
      "key_up_order": "strict_inverse",
      "key_up_when": "any",
      "to_after_key_up": [
        {
          "set_variable": {
            "name": "simlayer_active",
            "value": 0
          }
        }
      ]
    }
  },
  "to": [
    {
      "key_code": "escape"
    }
  ]
}
```

#### Simultaneous Options

The `simultaneous_options` object controls how simultaneous key presses are detected:

| Option | Values | Description |
|--------|--------|-------------|
| `detect_key_down_uninterruptedly` | true/false | If true, keys must be pressed without any interruptions from other keys |
| `key_down_order` | "strict", "strict_inverse", "insensitive" | Controls the order keys must be pressed down |
| `key_up_order` | "strict", "strict_inverse", "insensitive" | Controls the order keys must be released |
| `key_up_when` | "any", "all" | When to trigger key up events - after any key is released or all keys |
| `to_after_key_up` | array | Events to trigger after the simultaneous keys are released |

**Key Order Values:**
- `"strict"` - Keys must be pressed in the exact order specified
- `"strict_inverse"` - Keys must be pressed in reverse order
- `"insensitive"` - Keys can be pressed in any order

**Common Use Cases:**
- **Simlayers**: Use a key as a layer modifier (e.g., hold 'd' + press other keys)
- **Chord triggers**: Press multiple keys together to trigger an action
- **Variable management**: Set/unset variables to track layer states

### Conditional Mappings

Apply mappings only under certain conditions:

```json
{
  "conditions": [
    {
      "type": "frontmost_application_if",
      "bundle_identifiers": ["^com\\.apple\\.Terminal$"]
    }
  ]
}
```

## File Location

The configuration file is located at:
- macOS: `~/.config/karabiner/karabiner.json`

Karabiner-Elements watches this file for changes and automatically reloads the configuration when it's modified.