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

import "modulepath:/karabiner.pkl" as karabiner
import "modulepath:/helpers.pkl" as helpers
import "modulepath:/yabai.pkl" as yabai

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

#[test]
fn test_yabai_auto_mapping() {
    let ctx = TestContext::new();
    let fixture_content = TestContext::load_fixture("yabai_auto_test.pkl");

    let pkl_file = ctx.write_pkl_file("yabai_auto_test.pkl", &fixture_content);
    let result = ctx
        .compile_pkl_sync(&pkl_file, None)
        .expect("Failed to compile yabai auto fixture");

    // Verify the fixture compiles and produces expected output
    assert_eq!(result["profiles"][0]["name"], "pkl");

    let rules = &result["profiles"][0]["complex_modifications"]["rules"];
    let rules_array = rules.as_array().unwrap();

    // Should have 2 rules: one for focus spaces, one for quick navigation
    assert_eq!(rules_array.len(), 2);

    // Check focus spaces rule
    let focus_space_rule = &rules_array[0];
    assert_eq!(focus_space_rule["description"], "Yabai: Focus spaces");

    let focus_manipulators = focus_space_rule["manipulators"].as_array().unwrap();
    // Should have manipulators for u, i, o, p, [, ] keys
    assert_eq!(focus_manipulators.len(), 6);

    // First manipulator should be spacebar+u -> focus space 1
    let first_focus = &focus_manipulators[0];
    assert_eq!(
        first_focus["from"]["simultaneous"][0]["key_code"],
        "spacebar"
    );
    assert_eq!(first_focus["from"]["simultaneous"][1]["key_code"], "u");
    assert!(first_focus["to"][0]["shell_command"]
        .as_str()
        .unwrap()
        .contains("space --focus 1"));

    // Check quick navigation rule
    let nav_rule = &rules_array[1];
    assert_eq!(nav_rule["description"], "Yabai: Quick navigation");

    let nav_manipulators = nav_rule["manipulators"].as_array().unwrap();
    // Should have 4 manipulators: prevSpace, nextSpace, focusRecentWindow, focusRecentSpace
    assert_eq!(nav_manipulators.len(), 4);

    // Check hyperkey mapping (Cmd+Ctrl+Opt+Shift+H)
    let prev_space_manipulator = &nav_manipulators[0];
    assert_eq!(prev_space_manipulator["from"]["key_code"], "h");
    let modifiers = prev_space_manipulator["from"]["modifiers"]["mandatory"]
        .as_array()
        .unwrap();
    assert_eq!(modifiers.len(), 4); // All 4 modifiers for hyperkey
    assert!(prev_space_manipulator["to"][0]["shell_command"]
        .as_str()
        .unwrap()
        .contains("space --focus prev"));
}

#[test]
fn test_yabai_enhanced_module() {
    let ctx = TestContext::new();
    let fixture_content = TestContext::load_fixture("yabai_enhanced_test.pkl");

    let pkl_file = ctx.write_pkl_file("yabai_enhanced_test.pkl", &fixture_content);
    let json = ctx
        .compile_pkl_sync(&pkl_file, None)
        .expect("Failed to compile PKL");

    // Check that profile exists and has rules
    assert_eq!(json["profiles"][0]["name"], "pkl");
    let rules = &json["profiles"][0]["complex_modifications"]["rules"];
    let rules = rules.as_array().unwrap();
    assert!(!rules.is_empty(), "No rules generated");

    // Check for minimal configuration rule
    let minimal_rules: Vec<_> = rules
        .iter()
        .filter(|r| r["description"] == "Yabai: Focus windows (command+option)")
        .collect();
    assert_eq!(
        minimal_rules.len(),
        1,
        "Should have minimal focus window rule"
    );

    // Check for complete configuration rules
    let complete_rules: Vec<_> = rules
        .iter()
        .filter(|r| {
            let desc = r["description"].as_str().unwrap_or("");
            desc.starts_with("Yabai:")
        })
        .collect();
    assert!(
        complete_rules.len() > 10,
        "Should have many rules from complete config"
    );

    // Check for hyper key configuration
    let hyper_rules: Vec<_> = rules
        .iter()
        .filter(|r| {
            let desc = r["description"].as_str().unwrap_or("");
            desc.contains("command+option+control+shift")
        })
        .collect();
    assert!(!hyper_rules.is_empty(), "Should have hyper key rules");

    // Check for arrow key configuration
    let arrow_rule = rules
        .iter()
        .find(|r| r["description"] == "Yabai: Focus windows (command)")
        .expect("Should have arrow key focus rule");

    let manipulators = arrow_rule["manipulators"].as_array().unwrap();
    let has_arrow_keys = manipulators.iter().any(|m| {
        let key_code = m["from"]["key_code"].as_str().unwrap_or("");
        key_code.contains("arrow")
    });
    assert!(has_arrow_keys, "Should have arrow key manipulators");

    // Verify manipulator structure
    for rule in rules {
        if rule["description"]
            .as_str()
            .unwrap_or("")
            .starts_with("Yabai:")
        {
            assert!(rule["manipulators"].is_array());
            let manipulators = rule["manipulators"].as_array().unwrap();

            for manipulator in manipulators {
                assert_eq!(manipulator["type"], "basic");
                assert!(manipulator["from"]["key_code"].is_string());
                assert!(manipulator["to"].is_array());

                let to_events = manipulator["to"].as_array().unwrap();
                assert!(!to_events.is_empty());

                for event in to_events {
                    if event["shell_command"].is_string() {
                        let cmd = event["shell_command"].as_str().unwrap();
                        assert!(cmd.contains("yabai"), "Shell command should contain yabai");
                    }
                }
            }
        }
    }

    // Check specific command generation
    let mut all_shell_commands = Vec::new();
    for rule in rules {
        if let Some(manipulators) = rule["manipulators"].as_array() {
            for manipulator in manipulators {
                if let Some(to_events) = manipulator["to"].as_array() {
                    for event in to_events {
                        if let Some(cmd) = event["shell_command"].as_str() {
                            all_shell_commands.push(cmd.to_string());
                        }
                    }
                }
            }
        }
    }

    // Verify various yabai commands are present
    assert!(all_shell_commands
        .iter()
        .any(|c| c.contains("--focus west")));
    assert!(all_shell_commands.iter().any(|c| c.contains("--swap")));
    assert!(all_shell_commands.iter().any(|c| c.contains("--warp")));
    assert!(all_shell_commands.iter().any(|c| c.contains("--space")));
    assert!(all_shell_commands.iter().any(|c| c.contains("--display")));
    assert!(all_shell_commands
        .iter()
        .any(|c| c.contains("--toggle float")));
    assert!(all_shell_commands.iter().any(|c| c.contains("--layout")));
    assert!(all_shell_commands.iter().any(|c| c.contains("--resize")));
    assert!(all_shell_commands.iter().any(|c| c.contains("--ratio")));
    assert!(all_shell_commands.iter().any(|c| c.contains("--insert")));
    assert!(all_shell_commands.iter().any(|c| c.contains("--move rel")));
    assert!(all_shell_commands.iter().any(|c| c.contains("--grid")));
}
