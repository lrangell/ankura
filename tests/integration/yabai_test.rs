use crate::helpers::TestContext;

#[test]
fn test_yabai_fixture() {
    let ctx = TestContext::new();
    let fixture_content = TestContext::load_fixture("yabai_test.pkl");

    let pkl_file = ctx.write_pkl_file("yabai_test.pkl", &fixture_content);
    let result = ctx
        .compile_pkl_sync(&pkl_file, None)
        .expect("Failed to compile yabai fixture");

    // Verify the fixture compiles and produces expected output
    assert_eq!(result["profiles"][0]["name"], "pkl");

    let rules = &result["profiles"][0]["complex_modifications"]["rules"];
    assert!(!rules.as_array().unwrap().is_empty());

    // Check that spacebar simlayer rule was created
    let spacebar_rule = &rules[0];
    assert_eq!(spacebar_rule["description"], "Simlayer: spacebar + key");

    // Verify yabai shell commands are generated correctly
    let manipulators = spacebar_rule["manipulators"].as_array().unwrap();
    assert!(!manipulators.is_empty());

    // Check specific key bindings
    let first_manipulator = &manipulators[0];
    assert_eq!(
        first_manipulator["from"]["simultaneous"][0]["key_code"],
        "spacebar"
    );
    assert!(first_manipulator["to"][0]["shell_command"]
        .as_str()
        .unwrap()
        .contains("yabai"));
}

#[test]
fn test_yabai_helpers() {
    let ctx = TestContext::new();

    let pkl_content = r#"
module test

import "modulepath:/karabiner_pkl/lib/karabiner.pkl" as karabiner
import "modulepath:/karabiner_pkl/lib/helpers.pkl" as helpers
import "modulepath:/karabiner_pkl/lib/yabai.pkl" as yabai

// Test the new helper functions
testYabai: yabai.Yabai = new {
  cmdPath = "/usr/local/bin/yabai"
}

// Test simlayerKeys (returns key->index mapping)
testIndexMapping: Mapping<String, Int> = helpers.simlayerKeys("spacebar", List("a", "b", "c"))

// Test hyperkey
testHyperkey: karabiner.ToEvent = helpers.hyperkey("h")

// Test ctrl
testCtrl: karabiner.ToEvent = helpers.ctrl("n")

// Test range
testRange: List<String> = helpers.range(1, 5)

// Test qwertyRange
testQwertyRange: List<String> = helpers.qwertyRange("q", "t")

simpleConfig: karabiner.SimpleConfig = new {
  complex_modifications = new karabiner.ComplexModifications {
    rules = List(
      // Test simlayer with hyperkey and ctrl mappings
      helpers.simlayer("spacebar", new Mapping<String, String | karabiner.ToEvent> {
        ["h"] = testHyperkey
        ["n"] = testCtrl
        ["a"] = "1"
        ["b"] = "2"
        ["c"] = "3"
      }, null)
    )
  }
}

config: karabiner.Config = simpleConfig.toConfig()
"#;

    let pkl_file = ctx.write_pkl_file("yabai_helpers_test.pkl", pkl_content);
    let result = ctx
        .compile_pkl_sync(&pkl_file, None)
        .expect("Failed to compile");

    // Verify the simlayer rule was created
    let rules = &result["profiles"][0]["complex_modifications"]["rules"];
    let spacebar_rule = &rules[0];

    let manipulators = spacebar_rule["manipulators"].as_array().unwrap();
    assert_eq!(manipulators.len(), 5); // Should have 5 manipulators for keys h, n, a, b, c

    // Check that hyperkey mapping works (spacebar+h)
    let hyperkey_manipulator = manipulators
        .iter()
        .find(|m| m["from"]["simultaneous"][1]["key_code"] == "h")
        .unwrap();
    assert_eq!(hyperkey_manipulator["to"][0]["key_code"], "h");
    assert_eq!(
        hyperkey_manipulator["to"][0]["modifiers"]
            .as_array()
            .unwrap()
            .len(),
        4
    );

    // Check that ctrl mapping works (spacebar+n)
    let ctrl_manipulator = manipulators
        .iter()
        .find(|m| m["from"]["simultaneous"][1]["key_code"] == "n")
        .unwrap();
    assert_eq!(ctrl_manipulator["to"][0]["key_code"], "n");
    assert_eq!(ctrl_manipulator["to"][0]["modifiers"][0], "left_control");
}
