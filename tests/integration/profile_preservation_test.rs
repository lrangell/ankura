use crate::helpers::*;
use karabiner_pkl::cli::{merge_configurations, write_karabiner_config};
use karabiner_pkl::compiler::Compiler;

#[test]
fn test_preserves_other_profiles() {
    let ctx = TestContext::new();

    // Create a karabiner.json with multiple profiles
    let existing_karabiner = r#"{
        "profiles": [
            {
                "name": "Work",
                "selected": false,
                "complex_modifications": {
                    "rules": [
                        {
                            "description": "Work specific rule",
                            "manipulators": []
                        }
                    ]
                }
            },
            {
                "name": "Gaming",
                "selected": false,
                "simple_modifications": [
                    {
                        "from": "caps_lock",
                        "to": "left_control"
                    }
                ]
            },
            {
                "name": "pkl",
                "selected": true,
                "complex_modifications": {
                    "rules": []
                }
            }
        ]
    }"#;

    // Create test output directory
    let output_dir = ctx.temp_dir.path().join("output");
    std::fs::create_dir_all(&output_dir).unwrap();
    let output_file = output_dir.join("test_karabiner.json");

    // Write the existing karabiner.json
    std::fs::write(&output_file, existing_karabiner).unwrap();

    // Create a simple pkl config that updates the "pkl" profile
    let pkl_content = r#"
import "modulepath:/karabiner_pkl/lib/karabiner.pkl"

config = new karabiner.SimpleConfig {
  profileName = "pkl"
  simple_modifications = List(
    new karabiner.SimpleModification {
      from = "escape"
      to = "caps_lock"
    }
  )
}.toConfig()
"#;

    let pkl_file = ctx.write_pkl_file("update_profile.pkl", pkl_content);

    // Compile and merge configuration
    let compiler = Compiler::new().expect("Failed to create compiler");
    let compiled_config =
        tokio_test::block_on(compiler.compile(&pkl_file, None)).expect("Compile should succeed");

    let final_config =
        merge_configurations(&output_file, compiled_config).expect("Merge should succeed");

    write_karabiner_config(&output_file, &final_config).expect("Write should succeed");

    // Read the updated file
    let updated_content = std::fs::read_to_string(&output_file).unwrap();
    let updated_json: serde_json::Value = serde_json::from_str(&updated_content).unwrap();

    // Verify all profiles are still there
    let profiles = updated_json["profiles"].as_array().unwrap();
    assert_eq!(profiles.len(), 3, "Should still have 3 profiles");

    // Verify Work profile is unchanged
    let work_profile = profiles.iter().find(|p| p["name"] == "Work").unwrap();
    assert_eq!(work_profile["selected"], serde_json::json!(false));
    assert_eq!(
        work_profile["complex_modifications"]["rules"][0]["description"],
        "Work specific rule"
    );

    // Verify Gaming profile is unchanged
    let gaming_profile = profiles.iter().find(|p| p["name"] == "Gaming").unwrap();
    assert_eq!(gaming_profile["selected"], serde_json::json!(false));
    assert_eq!(
        gaming_profile["simple_modifications"][0]["from"],
        "caps_lock"
    );

    // Verify pkl profile was updated
    let pkl_profile = profiles.iter().find(|p| p["name"] == "pkl").unwrap();
    assert_eq!(pkl_profile["selected"], serde_json::json!(true));
    assert_eq!(pkl_profile["simple_modifications"][0]["from"], "escape");
}

#[test]
fn test_creates_new_profile_if_not_exists() {
    let ctx = TestContext::new();

    // Create a karabiner.json without our target profile
    let existing_karabiner = r#"{
        "profiles": [
            {
                "name": "Default",
                "selected": true,
                "complex_modifications": {
                    "rules": []
                }
            }
        ]
    }"#;

    // Create test output directory
    let output_dir = ctx.temp_dir.path().join("output");
    std::fs::create_dir_all(&output_dir).unwrap();
    let output_file = output_dir.join("test_karabiner.json");

    // Write the existing karabiner.json
    std::fs::write(&output_file, existing_karabiner).unwrap();

    // Create a pkl config with a custom profile name
    let pkl_content = r#"
import "modulepath:/karabiner_pkl/lib/karabiner.pkl"

config = new karabiner.SimpleConfig {
  profileName = "MyNewProfile"
  simple_modifications = List()
}.toConfig()
"#;

    let pkl_file = ctx.write_pkl_file("new_profile.pkl", pkl_content);

    // Compile and merge configuration
    let compiler = Compiler::new().expect("Failed to create compiler");
    let compiled_config =
        tokio_test::block_on(compiler.compile(&pkl_file, None)).expect("Compile should succeed");

    let final_config =
        merge_configurations(&output_file, compiled_config).expect("Merge should succeed");

    write_karabiner_config(&output_file, &final_config).expect("Write should succeed");

    // Read the updated file
    let updated_content = std::fs::read_to_string(&output_file).unwrap();
    let updated_json: serde_json::Value = serde_json::from_str(&updated_content).unwrap();

    // Verify both profiles exist
    let profiles = updated_json["profiles"].as_array().unwrap();
    assert_eq!(profiles.len(), 2, "Should have 2 profiles");

    // Verify Default profile is unchanged
    let default_profile = profiles.iter().find(|p| p["name"] == "Default").unwrap();
    assert_eq!(default_profile["selected"], serde_json::json!(true));

    // Verify new profile was created
    let new_profile = profiles
        .iter()
        .find(|p| p["name"] == "MyNewProfile")
        .unwrap();
    assert_eq!(new_profile["selected"], serde_json::json!(false));
}

#[test]
fn test_cli_profile_override_with_output() {
    let ctx = TestContext::new();

    // Create test output directory
    let output_dir = ctx.temp_dir.path().join("output");
    std::fs::create_dir_all(&output_dir).unwrap();
    let output_file = output_dir.join("test_override.json");

    // Create a pkl config
    let pkl_content = r#"
import "modulepath:/karabiner_pkl/lib/karabiner.pkl"

config = new karabiner.SimpleConfig {
  profileName = "ConfigProfile"
  simple_modifications = List()
}.toConfig()
"#;

    let pkl_file = ctx.write_pkl_file("override_test.pkl", pkl_content);

    // Compile with profile override
    let compiler = Compiler::new().expect("Failed to create compiler");
    let compiled_config = tokio_test::block_on(compiler.compile(&pkl_file, Some("CLIProfile")))
        .expect("Compile should succeed");

    write_karabiner_config(&output_file, &compiled_config).expect("Write should succeed");

    // Read the output file
    let content = std::fs::read_to_string(&output_file).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    // Verify CLI override took effect
    assert_eq!(json["profiles"][0]["name"], "CLIProfile");
}
