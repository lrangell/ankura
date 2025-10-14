# ankura

A type-safe configuration tool for [Karabiner-Elements](https://karabiner-elements.pqrs.org/) using Apple's [Pkl](https://pkl-lang.org/) configuration language.

ankura brings the power of [Pkl](https://pkl-lang.org/) to [Karabiner-Elements](https://karabiner-elements.pqrs.org/) configuration, allowing you to define keymaps in a declarative manner. [Pkl](https://pkl-lang.org/) provides autocomplete, type safety, validation, and excellent editor support that catches errors as you type. As a full programming language, Pkl lets you create your own abstractions and reusable patterns. You write simple, readable Pkl configurations that compile to Karabiner JSON. The live-reload daemon applies your changes instantly, making keyboard customization feel natural instead of painful.

**Features:**
- **Type-safe declarative DSL** - Write keyboard configurations in [Pkl](https://pkl-lang.org/) with full type checking, validation, and IDE support instead of error-prone JSON
- **Built-in helpers for common patterns** - Pre-built abstractions for hyper keys, symbol layers, simultaneous key chords, dual-use keys, and sticky modifiers
- **First-class macOS integrations** - Native support for popular window managers (yabai, AeroSpace) and system automation through shell commands
- **Live-reload daemon** - File watching with instant configuration updates and desktop notifications when changes are applied

## Table of Contents

- [Installation](#installation)
- [Usage](#usage)
  - [Dual-Use Keys](#dual-use-keys)
  - [Simultaneous Layers](#simultaneous-layers)
  - [Layers](#layers)
  - [Simple Modifications](#simple-modifications)
  - [Actions](#actions)
  - [Built-ins](#built-ins)
- [Yabai Integration](#yabai-integration)
- [AeroSpace Integration](#aerospace-integration)

## Installation

```bash
brew install lrangell/ankura
```

## Usage

Create or edit `~/.config/ankura.pkl`:

```pkl
extends "/opt/homebrew/var/lib/ankura/config.pkl"

name = "My Config"
keys = new Keys{}
actions = new Actions {}
mods = new Modifiers {}

rules = List(
  // Your rules go here
)
```

### Layers

Create modal layers for complex workflows.

```pkl
Layer {
  modifier = mods.ctrl
  h = keys.left
  j = keys.down
  k = keys.up
  l = keys.right
}
```

### Dual-Use Keys

Transform keys to behave differently when tapped vs held.

```pkl
DualUse {
  key = keys.capsLock
  tap = keys.esc
  hold = keys.ctrl
}
```

### Simultaneous Layers

Activate arrow keys while holding a trigger.

```pkl
SimLayer {
  trigger = "f"
  h = keys.left
  j = keys.down
  k = keys.up
  l = keys.right
}
```

### Simple Modifications

Basic key remapping.

```pkl
simpleModifications = List(
  new SimpleModification {
    from = "right_command"
    to = "right_option"
  }
)
```

### SpaceMode

Space key + other keys for quick shortcuts.

```pkl
SpaceMode {
  h = keys.left
  j = keys.down
  k = keys.up
  l = keys.right
  comma = keys.option.and(keys.comma)
}
```

### Actions

Trigger app launches, text insertion, and shell commands.

```pkl
SpaceMode {
  w = actions.launchApp("Google Chrome")
  s = actions.focusOrLaunchApp("Slack")
  u = actions.typeText("Hello, World!")
  n = actions.showNotification("Title", "Message")
}
```

### Built-ins

Built-in patterns for common keyboard customizations.

```pkl
List(
  builtins.hyperKey(),              // caps_lock → ⌃⌥⇧⌘
  builtins.hyperKeyDualUse(),       // caps_lock → ⌃⌥⇧⌘ when held, escape when tapped

  // Symbol layer for programming
  builtins.symbolLayer(keys.rightShift.key_code),  // hold right_shift for symbols

  // Simultaneous key combos (chords)
  builtins.simultaneous("j", "k", keys.escape),     // j+k → escape

  // Key swaps
  builtins.swapKeys(keys.tab.key_code, keys.escape.key_code),     // swap any two keys
  builtins.swapSemicolon(),                  // ; ↔ : swap
  builtins.swapGraveEscape(),                // ` ↔ escape
  builtins.swapBackslashPipe(),              // \ ↔ | swap
  builtins.swapCapsCtrl(),                   // capsLock ↔ leftControl
  builtins.swapCmdOpt(),                     // leftCommand ↔ leftOption

  // Sticky modifiers (one-shot)
  builtins.stickyModifier(keys.f.key_code, mods.leftShift),  // tap f → next key shifted
  builtins.stickyShift(),                    // tap shift → next key shifted
  builtins.stickyCmd(),                      // tap cmd → next key with cmd
  builtins.stickyOpt(),                      // tap opt → next key with opt
  builtins.stickyCtrl()                      // tap ctrl → next key with ctrl
)
```

**Custom options:**

```pkl
List(
  builtins.hyperKey(keys.tab.key_code),                    // custom trigger key
  builtins.simultaneous("f", "d", keys.deleteForward, 30), // custom threshold
  builtins.symbolLayer(keys.capsLock.key_code, Mapping {
    ["h"] = keys.openBracket     // h → [
    ["j"] = keys.closeBracket    // j → ]
  }),
  builtins.stickyShift(keys.f.key_code)                   // custom trigger for sticky shift
)
```

## Yabai Integration

<details>
<summary>Click to expand yabai window management examples</summary>

Control yabai window manager directly from your keyboard:

```pkl
yabai {
  modifier = "d"
  window {
    focus {
      west = "h"
      south = "j"
      north = "k"
      east = "l"
    }

    swap {
      modifier = List(mods.cmd, mods.shift)
      west = "h"
      south = "j"
      north = "k"
      east = "l"
    }

    resize {
      modifier = List(mods.ctrl, mods.opt)
      left = "h"
      down = "j"
      up = "k"
      right = "l"
    }
  }

  space {
    focus {
      mappings {
        ["1"] = "u"
        ["2"] = "i"
        ["3"] = "o"
        ["4"] = "p"
        ["5"] = keys.openBracket
      }
      prev = "n"
      next = "m"
    }
  }

  display {
    focus {
      modifier = List(mods.cmd, mods.opt)
      mappings {
        ["1"] = "1"
        ["2"] = "2"
        ["3"] = "3"
      }
      prev = keys.comma
      next = keys.period
    }
  }

  toggles {
    modifier = List(mods.cmd, mods.opt, mods.shift)
    float = "f"
    fullscreen = "m"
    sticky = "s"
    zoom = "z"
  }
}
```

</details>

## AeroSpace Integration

<details>
<summary>Click to expand AeroSpace window management examples</summary>

Control AeroSpace tiling window manager with keyboard shortcuts:

```pkl
aerospace {
  modifier = "f"
  window {
    focus {
      left = "h"
      down = "j"
      up = "k"
      right = "l"
      dfsNext = "n"
      dfsPrev = "p"
    }

    move {
      left = "h"
      down = "j"
      up = "k"
      right = "l"
    }

    resize {
      modifier = List(mods.opt, mods.ctrl)
      width = "w"
      height = "h"
      smart = "s"
      amount = 100
    }

    layout {
      tiling = "t"
      floating = "f"
      fullscreen = "m"
    }
  }

  workspace {
    focus {
      mappings {
        ["1"] = "u"
        ["2"] = "i"
        ["3"] = "o"
      }
      next = keys.tab
      prev = "h"
    }

    move {
      modifier = List(mods.opt, mods.shift)
      mappings {
        ["1"] = "1"
        ["2"] = "2"
        ["3"] = "3"
      }
      prev = "n"
      next = "m"
    }
  }

  modes {
    resize {
      trigger = "r"
      modifier = mods.opt
      bindings {
        ["h"] = "resize width -50"
        ["l"] = "resize width +50"
        ["j"] = "resize height +50"
        ["k"] = "resize height -50"
      }
    }
  }
}
```

</details>
