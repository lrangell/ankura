# Pkl Types Reference

This document provides a comprehensive reference for the Pkl type system used in Karabiner-Pkl.

## Core Configuration Types

### Config
The root configuration object that Karabiner-Elements reads.

```pkl
class Config {
  title: String = "Karabiner-Pkl Configuration"
  profiles: List<Profile> = List(defaultProfile)
  global: Global?
}
```

### Global
Global Karabiner-Elements settings.

```pkl
class Global {
  check_for_updates_on_startup: Boolean = true
  show_in_menu_bar: Boolean = true
  show_profile_name_in_menu_bar: Boolean = false
}
```

### Profile
Individual keyboard configuration profile.

```pkl
class Profile {
  name: String
  selected: Boolean = false
  virtual_hid_keyboard: VirtualHidKeyboard?
  devices: List<Device> = List()
  fn_function_keys: List<FnFunctionKey> = List()
  simple_modifications: List<SimpleModification> = List()
  complex_modifications: ComplexModifications?
  parameters: Parameters?
}
```

## Device Configuration

### Device
Device-specific settings for keyboards and mice.

```pkl
class Device {
  identifiers: DeviceIdentifiers
  ignore: Boolean = false
  disable_built_in_keyboard_if_exists: Boolean = false
  fn_function_keys: List<FnFunctionKey> = List()
  simple_modifications: List<SimpleModification> = List()
  manipulate_caps_lock_led: Boolean = false
}
```

### DeviceIdentifiers
Hardware device identification.

```pkl
class DeviceIdentifiers {
  vendor_id: Int
  product_id: Int
  is_keyboard: Boolean = true
  is_pointing_device: Boolean = false
}
```

## Key Mapping Types

### SimpleModification
Basic one-to-one key remapping.

```pkl
class SimpleModification {
  from: KeyCode
  to: KeyCode
}
```

### ComplexModifications
Container for complex mapping rules.

```pkl
class ComplexModifications {
  parameters: ComplexModificationParameters?
  rules: List<Rule>
}
```

### Rule
A set of related key manipulations.

```pkl
class Rule {
  description: String
  manipulators: List<Manipulator>
}
```

### Manipulator
Core mapping logic with conditions and actions.

```pkl
class Manipulator {
  type: ManipulatorType = "basic"
  from: FromEvent
  to: List<ToEvent>?
  to_if_alone: List<ToEvent>?
  to_after_key_up: List<ToEvent>?
  to_delayed_action: DelayedAction?
  conditions: List<Condition>?
  parameters: ManipulatorParameters?
}
```

## Event Types

### FromEvent
Input trigger specification.

```pkl
class FromEvent {
  key_code: KeyCode?
  consumer_key_code: ConsumerKeyCode?
  pointing_button: PointingButton?
  modifiers: Modifiers?
  simultaneous: List<FromEvent>?
  simultaneous_options: SimultaneousOptions?
}
```

### ToEvent
Output action specification.

```pkl
class ToEvent {
  key_code: KeyCode?
  consumer_key_code: ConsumerKeyCode?
  pointing_button: PointingButton?
  shell_command: String?
  select_input_source: InputSource?
  set_variable: SetVariable?
  mouse_key: MouseKey?
  sticky_modifier: StickyModifier?
  modifiers: List<Modifier>?
  lazy: Boolean?
  repeat: Boolean?
  halt: Boolean?
  hold_down_milliseconds: Int?
}
```

## Modifier Types

### Modifiers
Modifier key requirements for triggers.

```pkl
class Modifiers {
  mandatory: List<Modifier>?
  optional: List<Modifier>?
}
```

### Modifier Values
Available modifier keys:

- `"caps_lock"`
- `"left_command"`, `"right_command"`
- `"left_control"`, `"right_control"`
- `"left_option"`, `"right_option"`
- `"left_shift"`, `"right_shift"`
- `"command"` (either left or right)
- `"control"` (either left or right)
- `"option"` (either left or right)
- `"shift"` (either left or right)
- `"any"` (any modifier)

## Simultaneous Keys

### SimultaneousOptions
Controls behavior of simultaneous key combinations.

```pkl
class SimultaneousOptions {
  detect_key_down_uninterruptedly: Boolean?
  key_down_order: "insensitive" | "strict" | "strict_inverse"?
  key_up_order: "insensitive" | "strict" | "strict_inverse"?
  key_up_when: "any" | "all"? = "any"
  to_after_key_up: List<ToEvent>?
}
```

## Conditions

### Condition
Contextual conditions for when rules apply.

```pkl
class Condition {
  type: ConditionType
  bundle_identifiers: List<String>?
  file_paths: List<String>?
  input_source_id: String?
  input_source_language: String?
  keyboard_types: List<KeyboardType>?
  name: String?
  value: Int | String | Boolean?
}
```

### ConditionType Values
- `"frontmost_application_if"`
- `"frontmost_application_unless"`
- `"device_if"`
- `"device_unless"`
- `"input_source_if"`
- `"input_source_unless"`
- `"variable_if"`
- `"variable_unless"`
- `"keyboard_type_if"`
- `"keyboard_type_unless"`

## Parameters

### Parameters
Profile-level timing parameters.

```pkl
class Parameters {
  delay_milliseconds_before_open_device: Int?
  "basic.to_if_alone_timeout_milliseconds": Int?
  "basic.to_if_held_down_threshold_milliseconds": Int?
  "basic.to_delayed_action_delay_milliseconds": Int?
  "basic.simultaneous_threshold_milliseconds": Int?
}
```

### ManipulatorParameters
Manipulator-specific timing overrides.

```pkl
class ManipulatorParameters {
  "basic.to_if_alone_timeout_milliseconds": Int?
  "basic.to_if_held_down_threshold_milliseconds": Int?
  "basic.to_delayed_action_delay_milliseconds": Int?
}
```

## SimpleConfig API

A simplified interface that abstracts away profile management.

```pkl
class SimpleConfig {
  simple_modifications: List<SimpleModification> = List()
  complex_modifications: ComplexModifications?
  parameters: Parameters?
  devices: List<Device> = List()
  
  function toConfig(): Config = new Config {
    profiles = List(new Profile {
      name = "Default"
      selected = true
      simple_modifications = simple_modifications
      complex_modifications = complex_modifications
      parameters = parameters
      devices = devices
    })
  }
}
```

## Type Aliases

```pkl
typealias KeyCode = String              // e.g., "a", "spacebar", "left_command"
typealias ConsumerKeyCode = String      // e.g., "play_or_pause", "volume_increment"
typealias PointingButton = String       // e.g., "button1", "button2"
typealias ManipulatorType = "basic" | "mouse_motion_to_scroll"
typealias KeyboardType = "ansi" | "iso" | "jis"
```

## Common Key Codes

### Letters and Numbers
- Letters: `"a"` through `"z"`
- Numbers: `"1"` through `"0"`

### Special Keys
- `"escape"`, `"return_or_enter"`, `"tab"`, `"spacebar"`
- `"hyphen"`, `"equal_sign"`, `"open_bracket"`, `"close_bracket"`
- `"backslash"`, `"semicolon"`, `"quote"`, `"grave_accent_and_tilde"`
- `"comma"`, `"period"`, `"slash"`

### Navigation
- `"up_arrow"`, `"down_arrow"`, `"left_arrow"`, `"right_arrow"`
- `"page_up"`, `"page_down"`, `"home"`, `"end"`
- `"delete_or_backspace"`, `"delete_forward"`

### Function Keys
- `"f1"` through `"f12"`

### Media Keys (Consumer Key Codes)
- `"play_or_pause"`, `"rewind"`, `"fast_forward"`
- `"volume_increment"`, `"volume_decrement"`, `"mute"`
- `"display_brightness_increment"`, `"display_brightness_decrement"`

## Usage Examples

### Simple Key Remapping
```pkl
simple_modifications = List(
  new SimpleModification {
    from = "caps_lock"
    to = "escape"
  }
)
```

### Complex Modifier Layer
```pkl
new Manipulator {
  from = new FromEvent {
    key_code = "h"
    modifiers = new Modifiers {
      mandatory = List("left_control")
    }
  }
  to = List(new ToEvent { key_code = "left_arrow" })
}
```

### Simultaneous Keys
```pkl
new Manipulator {
  from = new FromEvent {
    simultaneous = List(
      new FromEvent { key_code = "j" },
      new FromEvent { key_code = "k" }
    )
    simultaneous_options = new SimultaneousOptions {
      key_down_order = "strict"
    }
  }
  to = List(new ToEvent { key_code = "escape" })
}
```

### Conditional Mapping
```pkl
new Manipulator {
  from = new FromEvent { key_code = "e" }
  to = List(new ToEvent { key_code = "escape" })
  conditions = List(
    new Condition {
      type = "frontmost_application_if"
      bundle_identifiers = List("com.microsoft.VSCode")
    }
  )
}
```