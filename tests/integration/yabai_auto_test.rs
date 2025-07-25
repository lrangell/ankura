use crate::helpers::TestContext;

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
