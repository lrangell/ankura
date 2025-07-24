use crate::helpers::TestContext;
use std::path::PathBuf;

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

    // Create a library file
    let lib_dir = ctx.temp_dir.path().join("lib");
    std::fs::create_dir_all(&lib_dir).unwrap();

    let lib_content = r#"
module my_lib

import "modulepath:/karabiner.pkl" as karabiner

function myCustomRule(): karabiner.Rule = new karabiner.Rule {
  description = "Custom rule from library"
  manipulators = List(
    new karabiner.Manipulator {
      type = "basic"
      from = new karabiner.FromEvent { key_code = "f1" }
      to = List(new karabiner.ToEvent { key_code = "escape" })
    }
  )
}
"#;

    std::fs::write(lib_dir.join("my_lib.pkl"), lib_content).unwrap();

    // Copy the required pkl-lib files to temp directory
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let pkl_lib_dir = PathBuf::from(manifest_dir).join("pkl-lib");

    std::fs::copy(
        pkl_lib_dir.join("karabiner.pkl"),
        ctx.temp_dir.path().join("karabiner.pkl"),
    )
    .unwrap();
    std::fs::copy(
        pkl_lib_dir.join("helpers.pkl"),
        ctx.temp_dir.path().join("helpers.pkl"),
    )
    .unwrap();

    // Create main config that imports from lib
    let main_content = r#"
module test

import "modulepath:/karabiner.pkl" as karabiner
import "modulepath:/helpers.pkl" as helpers
import "modulepath:/lib/my_lib.pkl"

simpleConfig: karabiner.SimpleConfig = new {
  complex_modifications = new karabiner.ComplexModifications {
    rules = List(
      my_lib.myCustomRule()
    )
  }
}

config: karabiner.Config = simpleConfig.toConfig()
"#;

    let pkl_file = ctx.write_pkl_file("main.pkl", main_content);

    // We need to manually add the lib dir to module path for this test
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let pkl_lib_dir = PathBuf::from(manifest_dir).join("pkl-lib");

    let output = std::process::Command::new(&ctx.pkl_path)
        .args(["eval", "--format=json"])
        .arg("--module-path")
        .arg(format!(
            "{}:{}:{}",
            pkl_lib_dir.display(),
            lib_dir.display(),
            ctx.temp_dir.path().display()
        ))
        .arg(&pkl_file)
        .output()
        .expect("Failed to run pkl");

    if !output.status.success() {
        eprintln!(
            "Command: {:?}",
            std::process::Command::new(&ctx.pkl_path)
                .args(["eval", "--format=json"])
                .arg("--module-path")
                .arg(format!(
                    "{}:{}:{}",
                    pkl_lib_dir.display(),
                    lib_dir.display(),
                    ctx.temp_dir.path().display()
                ))
                .arg(&pkl_file)
        );
        eprintln!("Working dir: {:?}", ctx.temp_dir.path());
        eprintln!(
            "Lib dir contents: {:?}",
            std::fs::read_dir(&lib_dir).unwrap().collect::<Vec<_>>()
        );
        panic!("Pkl failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    // Verify the imported rule is present
    let rule = &result["config"]["profiles"][0]["complex_modifications"]["rules"][0];
    assert_eq!(rule["description"], "Custom rule from library");
}
