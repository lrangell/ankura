use crate::helpers::TestContext;

#[test]
fn test_space_mode() {
    let ctx = TestContext::new();
    
    let pkl_content = r#"
module test

import "karabiner.pkl" as karabiner
import "helpers.pkl" as helpers

simpleConfig: karabiner.SimpleConfig = new {
  complex_modifications = new karabiner.ComplexModifications {
    rules = List(
      helpers.spaceMode(new Mapping {
        ["h"] = helpers.left_arrow
        ["j"] = helpers.down_arrow
        ["k"] = helpers.up_arrow
        ["l"] = helpers.right_arrow
        ["d"] = helpers.delete_forward
        ["b"] = helpers.delete_or_backspace
      }, null)
    )
  }
}

config: karabiner.Config = simpleConfig.toConfig()
"#;
    
    let pkl_file = ctx.write_pkl_file("space_mode_test.pkl", pkl_content);
    let result = ctx.compile_pkl_to_json(&pkl_file).expect("Failed to compile");
    
    // Check that the rule was created correctly
    let rules = &result["config"]["profiles"][0]["complex_modifications"]["rules"];
    assert_eq!(rules[0]["description"], "Space Mode: Hold spacebar + key");
    
    let manipulators = &rules[0]["manipulators"];
    assert!(manipulators.as_array().unwrap().len() == 6);
    
    // Check first manipulator (space + h -> left arrow)
    let first = &manipulators[0];
    assert_eq!(first["type"], "basic");
    assert_eq!(first["from"]["simultaneous"][0]["key_code"], "spacebar");
    assert_eq!(first["from"]["simultaneous"][1]["key_code"], "h");
    assert_eq!(first["to"][0]["key_code"], "left_arrow");
    assert_eq!(first["from"]["simultaneous_options"]["key_down_order"], "strict");
}

#[test]
fn test_space_mode_with_custom_threshold() {
    let ctx = TestContext::new();
    
    let pkl_content = r#"
module test

import "karabiner.pkl" as karabiner
import "helpers.pkl" as helpers

simpleConfig: karabiner.SimpleConfig = new {
  complex_modifications = new karabiner.ComplexModifications {
    rules = List(
      helpers.spaceMode(
        new Mapping {
          ["a"] = "1"
          ["s"] = "2"
        },
        300  // custom threshold
      )
    )
  }
}

config: karabiner.Config = simpleConfig.toConfig()
"#;
    
    let pkl_file = ctx.write_pkl_file("space_mode_threshold_test.pkl", pkl_content);
    let result = ctx.compile_pkl_to_json(&pkl_file).expect("Failed to compile");
    
    let manipulator = &result["config"]["profiles"][0]["complex_modifications"]["rules"][0]["manipulators"][0];
    assert_eq!(manipulator["parameters"]["basic.simultaneous_threshold_milliseconds"], 300);
}