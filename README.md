# Karabiner-Pkl

A type-safe configuration tool for Karabiner-Elements using Apple's Pkl configuration language. Build sophisticated keyboard customizations with live-reload support and rich error reporting.

## Table of Contents

- [Installation](#installation)
- [Usage](#usage)
  - [Dual-Use Keys](#dual-use-keys)
  - [Simultaneous Layers](#simultaneous-layers)
  - [Key Layers](#key-layers)
  - [Simple Modifications](#simple-modifications)
- [Yabai Integration](#yabai-integration)
- [Advanced Usage](#advanced-usage)

## Installation

```bash
brew install lrangell/ankura
```

## Usage

Create or edit `~/.config/ankura.pkl`:

### Key Layers

Create modal layers for complex workflows.

````pkl
Layer {
    trigger = keys.spacebar
    mappings = Map(
        keys.h, keys.left_arrow,
        keys.j, keys.down_arrow,
        keys.k, keys.up_arrow,
        keys.l, keys.right_arrow
    )
}


### Dual-Use Keys

Transform keys to behave differently when tapped vs held.

```pkl
DualUse {
    key = keys.caps_lock
    tap = keys.escape
    hold = keys.left_control
}
````

### Simultaneous Layers

Activate arrow keys while holding a trigger.

```pkl
SimLayer {
    trigger = keys.f
    h = keys.left_arrow
    j = keys.down_arrow
    k = keys.up_arrow
    l = keys.right_arrow
}
```

### Simple Modifications

Basic key remapping.

```pkl
SimpleModification {
    from = keys.right_command
    to = keys.right_option
}
```

## Yabai Integration

<details>
<summary>Click to expand yabai window management examples</summary>

Control yabai window manager directly from your keyboard:

```pkl

name = "Yabai Profile"

yabaiConfig = new yabai.Yabai {
    window {
        modifier = keys.left_command

        focus {
            west = keys.h
            south = keys.j
            north = keys.k
            east = keys.l
            recent = keys.tab
        }

        swap {
            modifier = List(keys.left_command, keys.left_shift)
            west = keys.h
            south = keys.j
            north = keys.k
            east = keys.l
        }

        resize {
            modifier = List(keys.left_control, keys.left_option)
            left = keys.h
            down = keys.j
            up = keys.k
            right = keys.l
            increase = keys.plus
            decrease = keys.minus
            amount = 50
        }
    }

    space {
        focus {
            modifier = keys.left_command
            mappings = new Mapping {
                ["1"] = keys.key_1
                ["2"] = keys.key_2
                ["3"] = keys.key_3
                ["4"] = keys.key_4
                ["5"] = keys.key_5
            }
            prev = keys.open_bracket
            next = keys.close_bracket
        }
    }

    display {
        focus {
            modifier = List(keys.left_command, keys.left_option)
            mappings = new Mapping {
                ["1"] = keys.key_1
                ["2"] = keys.key_2
                ["3"] = keys.key_3
            }
            prev = keys.comma
            next = keys.period
        }
    }

    toggles {
        modifier = List(keys.left_command, keys.left_option, keys.left_shift)
        float = keys.f
        fullscreen = keys.m
        sticky = keys.s
        zoom = keys.z
    }
}
```

</details>

## AeroSpace Integration

<details>
<summary>Click to expand AeroSpace window management examples</summary>

Control AeroSpace tiling window manager with keyboard shortcuts:

```pkl

aerospace  {
    window {
        modifier = "left_alt"
        focus {
            left = "h"
            down = "j"
            up = "k"
            right = "l"
            dfsNext = "n"
            dfsPrev = "p"
        }

        move {
            // When mofifier is ommitted, it defaults to parent modifier
            left = "h"
            down = "j"
            up = "k"
            right = "l"
        }

        resize {
            modifier = List("left_alt", "left_control")
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
            modifier = "left_alt"
            mappings = {
                ["1"] = "1"
                ["2"] = "2"
                ["3"] = "3"
                ["web"] = "w"
                ["code"] = "c"
            }
            next = "tab"
            prev = "grave_accent_and_tilde"
        }

        move {
            modifier = List("left_alt", "left_shift")
            mappings = {
                ["1"] = "1"
                ["2"] = "2"
                ["3"] = "3"
            }
        }
    }

    // AeroSpace-specific: Mode system for contextual bindings
    modes {
        resize {
            trigger = "r"
            modifier = "left_alt"
            bindings {
                bindings = {
                    ["h"] = "resize width -50"
                    ["l"] = "resize width +50"
                    ["j"] = "resize height +50"
                    ["k"] = "resize height -50"
                }
            }
        }
    }
}
```

</details>

## Advanced Usage

<details>
<summary>Click to expand advanced customization examples</summary>

### Complex Conditional Rules

```pkl

rules = List(
    k.rule("Context-Aware Navigation", List(
        k.manipulator(
            k.map(k.key("h", Set(keys.left_command))),
            k.to(keys.left_arrow)
        ) { conditions = List(k.frontmostApp("com.apple.finder")) },

        k.manipulator(
            k.map(k.key("h", Set(keys.left_command))),
            k.to(keys.backspace)
        ) { conditions = List(k.frontmostApp("com.microsoft.VSCode")) }
    ))
)
```

### Custom Settings and Device Targeting

```pkl
settings = new k.ComplexModificationParameters {
    basic = new k.BasicParameters {
        simultaneousThresholdMilliseconds = 50
        toDelayedActionDelayMilliseconds = 500
        toIfAloneTimeoutMilliseconds = 1000
    }
}

devices = List(
    new k.Device {
        identifiers = new k.DeviceIdentifiers {
            vendorId = 1452
            productId = 641
        }
        simpleModifications = List(
            new k.SimpleModification {
                from = keys.caps_lock
                to = keys.left_control
            }
        )
    }
)
```

### Multi-Modal Layer System

```pkl

rules = List(
    // Navigation mode
    new k.Layer {
        trigger = keys.semicolon
        mappings = Map(
            keys.h, keys.left_arrow,
            keys.j, keys.down_arrow,
            keys.k, keys.up_arrow,
            keys.l, keys.right_arrow,
            keys.u, k.key(keys.left_arrow, Set(keys.left_command)),
            keys.i, k.key(keys.right_arrow, Set(keys.left_command))
        )
    },

    // Window management mode
    new k.Layer {
        trigger = keys.w
        mappings = Map(
            keys.f, k.key(keys.return, Set(keys.left_command, keys.left_control)),
            keys.h, k.key(keys.left_arrow, Set(keys.left_command, keys.left_option)),
            keys.l, k.key(keys.right_arrow, Set(keys.left_command, keys.left_option))
        )
    },

    // Application launcher
    new k.Layer {
        trigger = keys.spacebar
        condition = k.held(Set(keys.left_command))
        mappings = Map(
            keys.t, k.launchApp("Terminal"),
            keys.b, k.launchApp("Safari"),
            keys.c, k.launchApp("Visual Studio Code")
        )
    }
)
```

</details>

