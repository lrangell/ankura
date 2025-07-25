use crate::helpers::TestContext;

#[test]
fn test_char_range() {
    let ctx = TestContext::new();

    let pkl_content = r#"
module test

import "modulepath:/karabiner_pkl/lib/karabiner.pkl" as karabiner
import "modulepath:/karabiner_pkl/lib/helpers.pkl" as helpers

// Test char range generation
letters = helpers.charRange("a", "e")
numbers = helpers.charRange("0", "5")

simpleConfig: karabiner.SimpleConfig = new {
  complex_modifications = new karabiner.ComplexModifications {
    rules = List(
      new karabiner.Rule {
        description = "Test Char Range: \(letters.join(","))"
        manipulators = letters.map((letter) ->
          new karabiner.Manipulator {
            type = "basic"
            from = new karabiner.FromEvent { key_code = letter }
            to = List(new karabiner.ToEvent { key_code = letter })
          }
        )
      }
    )
  }
}

config: karabiner.Config = simpleConfig.toConfig()
"#;

    let pkl_file = ctx.write_pkl_file("char_range_test.pkl", pkl_content);
    let result = ctx
        .compile_pkl_sync(&pkl_file, None)
        .expect("Failed to compile");

    let rule = &result["profiles"][0]["complex_modifications"]["rules"][0];
    assert_eq!(rule["description"], "Test Char Range: a,b,c,d,e");

    let manipulators = rule["manipulators"].as_array().unwrap();
    assert_eq!(manipulators.len(), 5);
    assert_eq!(manipulators[0]["from"]["key_code"], "a");
    assert_eq!(manipulators[4]["from"]["key_code"], "e");
}

#[test]
fn test_num_range() {
    let ctx = TestContext::new();

    let pkl_content = r#"
module test

import "modulepath:/karabiner_pkl/lib/karabiner.pkl" as karabiner
import "modulepath:/karabiner_pkl/lib/helpers.pkl" as helpers

// Test number range generation
nums = helpers.numRange(1, 5)

simpleConfig: karabiner.SimpleConfig = new {
  complex_modifications = new karabiner.ComplexModifications {
    rules = List(
      new karabiner.Rule {
        description = "Test Num Range: \(nums.join(","))"
        manipulators = nums.map((num) ->
          new karabiner.Manipulator {
            type = "basic"
            from = new karabiner.FromEvent { key_code = num }
            to = List(new karabiner.ToEvent { key_code = num })
          }
        )
      }
    )
  }
}

config: karabiner.Config = simpleConfig.toConfig()
"#;

    let pkl_file = ctx.write_pkl_file("num_range_test.pkl", pkl_content);
    let result = ctx
        .compile_pkl_sync(&pkl_file, None)
        .expect("Failed to compile");

    let rule = &result["profiles"][0]["complex_modifications"]["rules"][0];
    assert_eq!(rule["description"], "Test Num Range: 1,2,3,4,5");

    let manipulators = rule["manipulators"].as_array().unwrap();
    assert_eq!(manipulators.len(), 5);
    assert_eq!(manipulators[0]["from"]["key_code"], "1");
    assert_eq!(manipulators[4]["from"]["key_code"], "5");
}

#[test]
fn test_qwerty_sequence() {
    let ctx = TestContext::new();

    let pkl_content = r#"
module test

import "modulepath:/karabiner_pkl/lib/karabiner.pkl" as karabiner
import "modulepath:/karabiner_pkl/lib/helpers.pkl" as helpers

// Test QWERTY sequence generation
homeRowRight = helpers.qwertySequence("j", 4)  // j, k, l, ;

simpleConfig: karabiner.SimpleConfig = new {
  complex_modifications = new karabiner.ComplexModifications {
    rules = List(
      new karabiner.Rule {
        description = "QWERTY Sequence: \(homeRowRight.join(" "))"
        manipulators = homeRowRight.map((key) ->
          new karabiner.Manipulator {
            type = "basic"
            from = new karabiner.FromEvent { key_code = key }
            to = List(new karabiner.ToEvent { key_code = key })
          }
        )
      }
    )
  }
}

config: karabiner.Config = simpleConfig.toConfig()
"#;

    let pkl_file = ctx.write_pkl_file("qwerty_sequence_test.pkl", pkl_content);
    let result = ctx
        .compile_pkl_sync(&pkl_file, None)
        .expect("Failed to compile");

    let rule = &result["profiles"][0]["complex_modifications"]["rules"][0];
    assert_eq!(rule["description"], "QWERTY Sequence: j k l semicolon");

    let manipulators = rule["manipulators"].as_array().unwrap();
    assert_eq!(manipulators.len(), 4);
    assert_eq!(manipulators[0]["from"]["key_code"], "j");
    assert_eq!(manipulators[1]["from"]["key_code"], "k");
    assert_eq!(manipulators[2]["from"]["key_code"], "l");
    assert_eq!(manipulators[3]["from"]["key_code"], "semicolon");
}

#[test]
fn test_vim_home_row_navigation() {
    let ctx = TestContext::new();

    let pkl_content = r#"
module test

import "modulepath:/karabiner_pkl/lib/karabiner.pkl" as karabiner
import "modulepath:/karabiner_pkl/lib/helpers.pkl" as helpers

// Test vim home row navigation helper
vimNav = helpers.vimHomeRowNavigation()

simpleConfig: karabiner.SimpleConfig = new {
  complex_modifications = new karabiner.ComplexModifications {
    rules = List(
      helpers.layer("left_control", vimNav, null)
    )
  }
}

config: karabiner.Config = simpleConfig.toConfig()
"#;

    let pkl_file = ctx.write_pkl_file("vim_home_row_test.pkl", pkl_content);
    let result = ctx
        .compile_pkl_sync(&pkl_file, None)
        .expect("Failed to compile");

    let rule = &result["profiles"][0]["complex_modifications"]["rules"][0];
    let manipulators = rule["manipulators"].as_array().unwrap();

    // Find the h -> left_arrow mapping
    let h_mapping = manipulators
        .iter()
        .find(|m| m["from"]["key_code"] == "h")
        .unwrap();
    assert_eq!(h_mapping["to"][0]["key_code"], "left_arrow");

    // Find the j -> down_arrow mapping
    let j_mapping = manipulators
        .iter()
        .find(|m| m["from"]["key_code"] == "j")
        .unwrap();
    assert_eq!(j_mapping["to"][0]["key_code"], "down_arrow");
}
