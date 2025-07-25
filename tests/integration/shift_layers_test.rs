use crate::helpers::TestContext;

#[test]
fn test_shift_layer() {
    let ctx = TestContext::new();

    let pkl_content = r#"
module test

import "modulepath:/karabiner.pkl" as karabiner
import "modulepath:/helpers.pkl" as helpers

simpleConfig: karabiner.SimpleConfig = new {
  complex_modifications = new karabiner.ComplexModifications {
    rules = List(
      helpers.shiftLayer("semicolon")
    )
  }
}

config: karabiner.Config = simpleConfig.toConfig()
"#;

    let pkl_file = ctx.write_pkl_file("shift_layer_test.pkl", pkl_content);
    let result = ctx
        .compile_pkl_sync(&pkl_file, None)
        .expect("Failed to compile");

    let rule = &result["profiles"][0]["complex_modifications"]["rules"][0];
    assert_eq!(rule["description"], "Shift Layer: semicolon + key");

    // Should have manipulators for all letters and numbers
    let manipulators = rule["manipulators"].as_array().unwrap();
    assert!(manipulators.len() > 30); // At least a-z and 0-9

    // Check a specific mapping (semicolon + a -> A)
    let a_manipulator = manipulators
        .iter()
        .find(|m| m["from"]["simultaneous"][1]["key_code"] == "a")
        .expect("Should have 'a' mapping");

    assert_eq!(
        a_manipulator["from"]["simultaneous"][0]["key_code"],
        "semicolon"
    );
    assert_eq!(a_manipulator["from"]["simultaneous"][1]["key_code"], "a");
    assert_eq!(a_manipulator["to"][0]["key_code"], "a");
    assert_eq!(a_manipulator["to"][0]["modifiers"][0], "left_shift");
}

#[test]
fn test_multiple_shift_layers() {
    let ctx = TestContext::new();
    let fixture_content = TestContext::load_fixture("shift_layers_test.pkl");

    let pkl_file = ctx.write_pkl_file("fixture_shift_layers.pkl", &fixture_content);
    let result = ctx
        .compile_pkl_sync(&pkl_file, None)
        .expect("Failed to compile fixture");

    let rules = result["profiles"][0]["complex_modifications"]["rules"]
        .as_array()
        .unwrap();
    assert_eq!(rules.len(), 2); // semicolon and a shift layers

    assert_eq!(rules[0]["description"], "Shift Layer: semicolon + key");
    assert_eq!(rules[1]["description"], "Shift Layer: a + key");
}
