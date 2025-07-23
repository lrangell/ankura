use crate::helpers::TestContext;

#[test]
fn test_shell_commands() {
    let ctx = TestContext::new();
    let fixture_content = TestContext::load_fixture("shell_commands_test.pkl");
    
    let pkl_file = ctx.write_pkl_file("shell_commands.pkl", &fixture_content);
    let result = ctx.compile_pkl_to_json(&pkl_file).expect("Failed to compile");
    
    let rule = &result["config"]["profiles"][0]["complex_modifications"]["rules"][0];
    let manipulators = rule["manipulators"].as_array().unwrap();
    
    // Test yabai space focus command
    let yabai_manip = &manipulators[0];
    assert_eq!(yabai_manip["from"]["key_code"], "u");
    assert_eq!(yabai_manip["from"]["modifiers"]["mandatory"][0], "d");
    assert_eq!(
        yabai_manip["to"][0]["shell_command"],
        "/opt/homebrew/bin/yabai -m space --focus 1"
    );
    
    // Test left_command with to_if_alone shell command
    let cmd_manip = &manipulators[1];
    assert_eq!(cmd_manip["from"]["key_code"], "left_command");
    assert_eq!(cmd_manip["to"][0]["key_code"], "left_command");
    assert_eq!(
        cmd_manip["to_if_alone"][0]["shell_command"],
        "/opt/homebrew/bin/yabai -m space --focus recent"
    );
}

#[test]
fn test_app_switcher_with_shell_commands() {
    let ctx = TestContext::new();
    
    let pkl_content = r#"
module test

import "karabiner.pkl" as karabiner
import "helpers.pkl" as helpers

simpleConfig: karabiner.SimpleConfig = new {
  complex_modifications = new karabiner.ComplexModifications {
    rules = List(
      helpers.appSwitcher("left_command", new Mapping {
        ["s"] = "Slack"
        ["c"] = "Google Chrome"
        ["t"] = "Terminal"
      })
    )
  }
}

config: karabiner.Config = simpleConfig.toConfig()
"#;
    
    let pkl_file = ctx.write_pkl_file("app_switcher_test.pkl", pkl_content);
    let result = ctx.compile_pkl_to_json(&pkl_file).expect("Failed to compile");
    
    let rule = &result["config"]["profiles"][0]["complex_modifications"]["rules"][0];
    assert_eq!(rule["description"], "App Switcher: left_command + key");
    
    let manipulators = rule["manipulators"].as_array().unwrap();
    assert_eq!(manipulators.len(), 3);
    
    // Check Slack launcher
    let slack_manip = manipulators.iter()
        .find(|m| m["from"]["key_code"] == "s")
        .expect("Should have Slack mapping");
    
    assert!(slack_manip["to"][0]["shell_command"]
        .as_str()
        .unwrap()
        .contains("Slack"));
}