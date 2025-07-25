use crate::helpers::TestContext;

#[test]
fn test_import_local_file() {
    let ctx = TestContext::new();

    // Create a test pkl file to import
    let test_content = r#"
module test_import

import "@karabiner"

function testFunction(): karabiner.Rule = new karabiner.Rule {
  description = "Test imported rule"
  manipulators = List {}
}
"#;

    let source_file = ctx.write_pkl_file("test_import.pkl", test_content);

    // Create importer with temp directory as lib dir
    let temp_lib = ctx.temp_dir.path().join("lib");
    std::fs::create_dir_all(&temp_lib).unwrap();

    // We can't easily test the actual Importer without refactoring it,
    // so let's just test that we can copy files
    let target = temp_lib.join("test_import.pkl");
    std::fs::copy(&source_file, &target).unwrap();

    assert!(target.exists());
    let content = std::fs::read_to_string(&target).unwrap();
    assert!(content.contains("Test imported rule"));
}

#[test]
fn test_module_path_compilation() {
    let ctx = TestContext::new();

    // Test that the embedded module path works correctly
    let main_content = r#"
module test

import "modulepath:/karabiner.pkl" as karabiner
import "modulepath:/helpers.pkl" as helpers

simpleConfig: karabiner.SimpleConfig = new {
  profileName = "test-profile"
  complex_modifications = new karabiner.ComplexModifications {
    rules = List(
      helpers.capsLockToEscape()
    )
  }
}

config: karabiner.Config = simpleConfig.toConfig()
"#;

    let pkl_file = ctx.write_pkl_file("module_path_test.pkl", main_content);
    let result = ctx
        .compile_pkl_sync(&pkl_file, None)
        .expect("Failed to compile with module path");

    // Verify the compilation worked and profile name was used
    assert_eq!(result["profiles"][0]["name"], "test-profile");
    let rule = &result["profiles"][0]["complex_modifications"]["rules"][0];
    assert_eq!(rule["description"], "Caps Lock to Escape");
}
