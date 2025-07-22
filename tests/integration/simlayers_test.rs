use crate::helpers::TestContext;

#[test]
fn test_simple_layer() {
    let ctx = TestContext::new();
    
    let pkl_content = r#"
module test

import "karabiner.pkl" as karabiner
import "helpers.pkl" as helpers

simpleConfig: karabiner.SimpleConfig = new {
  complex_modifications = new karabiner.ComplexModifications {
    rules = List(
      helpers.layer("d", new Mapping {
        ["h"] = "left_arrow"
        ["j"] = "down_arrow"
        ["k"] = "up_arrow"
        ["l"] = "right_arrow"
      })
    )
  }
}

config: karabiner.Config = simpleConfig.toConfig()
"#;
    
    let pkl_file = ctx.write_pkl_file("layer_test.pkl", pkl_content);
    let result = ctx.compile_pkl_to_json(&pkl_file).expect("Failed to compile");
    
    let rule = &result["profiles"][0]["complex_modifications"]["rules"][0];
    assert_eq!(rule["description"], "Layer: d + key");
    
    // Check that manipulators were created for each key
    let manipulators = rule["manipulators"].as_array().unwrap();
    assert_eq!(manipulators.len(), 4);
    
    // Verify first manipulator (h -> left_arrow)
    assert_eq!(manipulators[0]["from"]["key_code"], "h");
    assert_eq!(manipulators[0]["from"]["modifiers"]["mandatory"][0], "d");
    assert_eq!(manipulators[0]["to"][0]["key_code"], "left_arrow");
}

#[test]
fn test_multiple_layers() {
    let ctx = TestContext::new();
    let fixture_content = TestContext::load_fixture("simlayers_test.pkl");
    
    let pkl_file = ctx.write_pkl_file("fixture_simlayers.pkl", &fixture_content);
    let result = ctx.compile_pkl_to_json(&pkl_file).expect("Failed to compile fixture");
    
    let rules = result["profiles"][0]["complex_modifications"]["rules"].as_array().unwrap();
    assert_eq!(rules.len(), 2); // D-mode and F-mode
    
    // Check parameters were set
    let params = &result["profiles"][0]["complex_modifications"]["parameters"];
    assert_eq!(params["basic.simultaneous_threshold_milliseconds"], 200);
}