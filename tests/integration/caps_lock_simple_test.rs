use crate::helpers::TestContext;

#[test]
fn test_caps_lock_to_escape_simple() {
    let ctx = TestContext::new();

    let pkl_content = r#"
module test

import "modulepath:/karabiner.pkl" as karabiner
import "modulepath:/helpers.pkl" as helpers

simpleConfig: karabiner.SimpleConfig = new {
  complex_modifications = new karabiner.ComplexModifications {
    rules = List(
      helpers.capsLockToEscape()
    )
  }
}

config: karabiner.Config = simpleConfig.toConfig()
"#;

    let pkl_file = ctx.write_pkl_file("caps_lock_simple_test.pkl", pkl_content);
    let result = ctx
        .compile_pkl_sync(&pkl_file, None)
        .expect("Failed to compile");

    // Check that the rule was created correctly
    let rules = &result["profiles"][0]["complex_modifications"]["rules"];
    assert_eq!(rules[0]["description"], "Caps Lock to Escape");

    let manipulator = &rules[0]["manipulators"][0];
    assert_eq!(manipulator["from"]["key_code"], "caps_lock");
    assert_eq!(manipulator["to"][0]["key_code"], "escape");
    assert!(manipulator.get("to_if_alone").is_none());
}

#[test]
fn test_caps_lock_to_modifier_default() {
    let ctx = TestContext::new();

    let pkl_content = r#"
module test

import "modulepath:/karabiner.pkl" as karabiner
import "modulepath:/helpers.pkl" as helpers

simpleConfig: karabiner.SimpleConfig = new {
  complex_modifications = new karabiner.ComplexModifications {
    rules = List(
      helpers.capsLockToModifier(null)  // default is right_control
    )
  }
}

config: karabiner.Config = simpleConfig.toConfig()
"#;

    let pkl_file = ctx.write_pkl_file("caps_lock_modifier_default_test.pkl", pkl_content);
    let result = ctx
        .compile_pkl_sync(&pkl_file, None)
        .expect("Failed to compile");

    let rules = &result["profiles"][0]["complex_modifications"]["rules"];
    assert_eq!(
        rules[0]["description"],
        "Caps Lock to right_control when held, Escape when alone"
    );

    let manipulator = &rules[0]["manipulators"][0];
    assert_eq!(manipulator["from"]["key_code"], "caps_lock");
    assert_eq!(manipulator["to"][0]["key_code"], "right_control");
    assert_eq!(manipulator["to_if_alone"][0]["key_code"], "escape");
}

#[test]
fn test_caps_lock_to_modifier_custom() {
    let ctx = TestContext::new();

    let pkl_content = r#"
module test

import "modulepath:/karabiner.pkl" as karabiner
import "modulepath:/helpers.pkl" as helpers

simpleConfig: karabiner.SimpleConfig = new {
  complex_modifications = new karabiner.ComplexModifications {
    rules = List(
      helpers.capsLockToModifier("left_command")
    )
  }
}

config: karabiner.Config = simpleConfig.toConfig()
"#;

    let pkl_file = ctx.write_pkl_file("caps_lock_modifier_custom_test.pkl", pkl_content);
    let result = ctx
        .compile_pkl_sync(&pkl_file, None)
        .expect("Failed to compile");

    let rules = &result["profiles"][0]["complex_modifications"]["rules"];
    assert_eq!(
        rules[0]["description"],
        "Caps Lock to left_command when held, Escape when alone"
    );

    let manipulator = &rules[0]["manipulators"][0];
    assert_eq!(manipulator["from"]["key_code"], "caps_lock");
    assert_eq!(manipulator["to"][0]["key_code"], "left_command");
    assert_eq!(manipulator["to_if_alone"][0]["key_code"], "escape");
}
