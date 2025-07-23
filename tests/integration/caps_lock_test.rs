use crate::helpers::TestContext;

#[test]
fn test_caps_lock_to_escape_control() {
    let ctx = TestContext::new();
    
    let pkl_content = r#"
module test

import "karabiner.pkl" as karabiner
import "helpers.pkl" as helpers

simpleConfig: karabiner.SimpleConfig = new {
  complex_modifications = new karabiner.ComplexModifications {
    rules = List(
      helpers.capsLockToEscapeControl()
    )
  }
}

config: karabiner.Config = simpleConfig.toConfig()
"#;
    
    let pkl_file = ctx.write_pkl_file("caps_lock_test.pkl", pkl_content);
    let result = ctx.compile_pkl_to_json(&pkl_file).expect("Failed to compile");
    
    // Check that the rule was created correctly
    let rules = &result["config"]["profiles"][0]["complex_modifications"]["rules"];
    assert_eq!(rules[0]["description"], "Caps Lock to Escape when alone, Control when held");
    
    let manipulator = &rules[0]["manipulators"][0];
    assert_eq!(manipulator["from"]["key_code"], "caps_lock");
    assert_eq!(manipulator["to"][0]["key_code"], "right_control");
    assert_eq!(manipulator["to_if_alone"][0]["key_code"], "escape");
}

#[test]
fn test_caps_lock_in_fixtures() {
    let ctx = TestContext::new();
    let fixture_content = TestContext::load_fixture("caps_lock_test.pkl");
    
    let pkl_file = ctx.write_pkl_file("fixture_caps_lock.pkl", &fixture_content);
    let result = ctx.compile_pkl_to_json(&pkl_file).expect("Failed to compile fixture");
    
    // Verify the fixture compiles and produces expected output
    assert_eq!(result["config"]["profiles"][0]["name"], "Default");
    assert!(result["config"]["profiles"][0]["complex_modifications"]["rules"].as_array().unwrap().len() > 0);
}