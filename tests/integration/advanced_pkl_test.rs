use crate::helpers::TestContext;

#[test]
fn test_anonymous_functions() {
    let ctx = TestContext::new();
    
    let pkl_content = r#"
module test

import "karabiner.pkl" as karabiner
import "helpers.pkl" as helpers

// Test anonymous functions with map
simpleConfig: karabiner.SimpleConfig = new {
  complex_modifications = new karabiner.ComplexModifications {
    rules = List(
      new karabiner.Rule {
        description = "Anonymous Function Test"
        manipulators = List("a", "b", "c").map((letter) ->
          // Anonymous function creating manipulators
          new karabiner.Manipulator {
            type = "basic"
            from = new karabiner.FromEvent { 
              key_code = letter 
              modifiers = new karabiner.Modifiers {
                mandatory = List("left_shift")
              }
            }
            to = List(
              new karabiner.ToEvent { 
                key_code = letter
                modifiers = List("left_command")
              }
            )
          }
        )
      }
    )
  }
}

config: karabiner.Config = simpleConfig.toConfig()
"#;
    
    let pkl_file = ctx.write_pkl_file("anonymous_functions_test.pkl", pkl_content);
    let result = ctx.compile_pkl_to_json(&pkl_file).expect("Failed to compile");
    
    let rule = &result["config"]["profiles"][0]["complex_modifications"]["rules"][0];
    let manipulators = rule["manipulators"].as_array().unwrap();
    assert_eq!(manipulators.len(), 3);
    
    // Check first manipulator
    assert_eq!(manipulators[0]["from"]["key_code"], "a");
    assert_eq!(manipulators[0]["from"]["modifiers"]["mandatory"][0], "left_shift");
    assert_eq!(manipulators[0]["to"][0]["key_code"], "a");
    assert_eq!(manipulators[0]["to"][0]["modifiers"][0], "left_command");
}

#[test]
fn test_let_expressions() {
    let ctx = TestContext::new();
    
    let pkl_content = r#"
module test

import "karabiner.pkl" as karabiner
import "helpers.pkl" as helpers

// Test let expressions
simpleConfig: karabiner.SimpleConfig = new {
  complex_modifications = new karabiner.ComplexModifications {
    rules = List(
      // Using let to create reusable values
      let (commonMods = List("left_command", "left_shift"))
      let (timeout = 500)
      new karabiner.Rule {
        description = "Let Expression Test"
        manipulators = List(
          new karabiner.Manipulator {
            type = "basic"
            from = new karabiner.FromEvent { 
              key_code = "a"
              modifiers = new karabiner.Modifiers {
                mandatory = commonMods
              }
            }
            to = List(new karabiner.ToEvent { key_code = "1" })
            parameters = new karabiner.ManipulatorParameters {
              `basic.to_if_alone_timeout_milliseconds` = timeout
            }
          }
        )
      }
    )
  }
}

config: karabiner.Config = simpleConfig.toConfig()
"#;
    
    let pkl_file = ctx.write_pkl_file("let_expressions_test.pkl", pkl_content);
    let result = ctx.compile_pkl_to_json(&pkl_file).expect("Failed to compile");
    
    let manipulator = &result["config"]["profiles"][0]["complex_modifications"]["rules"][0]["manipulators"][0];
    assert_eq!(manipulator["from"]["modifiers"]["mandatory"][0], "left_command");
    assert_eq!(manipulator["from"]["modifiers"]["mandatory"][1], "left_shift");
    assert_eq!(manipulator["parameters"]["basic.to_if_alone_timeout_milliseconds"], 500);
}

#[test]
fn test_complex_layer_with_classes() {
    let ctx = TestContext::new();
    
    let pkl_content = r#"
module test

import "karabiner.pkl" as karabiner
import "helpers.pkl" as helpers

// Test creating a custom class for layer configuration
class LayerConfig {
  trigger: String
  mappings: Mapping<String, String>
  description: String
  threshold: Int = 200
  
  function toRule(): karabiner.Rule = 
    let (triggerKey = trigger)
    let (thresholdValue = threshold)
    let (descriptionText = description)
    new karabiner.Rule {
      description = descriptionText
      manipulators = mappings.toMap().entries.map((entry) ->
        new karabiner.Manipulator {
          type = "basic"
          from = new karabiner.FromEvent {
            key_code = entry.key
            modifiers = new karabiner.Modifiers {
              mandatory = List(triggerKey)
            }
          }
          to = List(new karabiner.ToEvent { key_code = entry.value })
          parameters = new karabiner.ManipulatorParameters {
            `basic.simultaneous_threshold_milliseconds` = thresholdValue
          }
        }
      )
    }
}

// Create custom layer using the class
myLayer = new LayerConfig {
  trigger = "right_option"
  description = "Custom Layer with Class"
  threshold = 300
  mappings = new Mapping<String, String> {
    ["1"] = "f1"
    ["2"] = "f2"
    ["3"] = "f3"
  }
}

simpleConfig: karabiner.SimpleConfig = new {
  complex_modifications = new karabiner.ComplexModifications {
    rules = List(myLayer.toRule())
  }
}

config: karabiner.Config = simpleConfig.toConfig()
"#;
    
    let pkl_file = ctx.write_pkl_file("class_layer_test.pkl", pkl_content);
    let result = ctx.compile_pkl_to_json(&pkl_file).expect("Failed to compile");
    
    let rule = &result["config"]["profiles"][0]["complex_modifications"]["rules"][0];
    assert_eq!(rule["description"], "Custom Layer with Class");
    
    let manipulators = rule["manipulators"].as_array().unwrap();
    assert_eq!(manipulators.len(), 3);
    assert_eq!(manipulators[0]["from"]["modifiers"]["mandatory"][0], "right_option");
    assert_eq!(manipulators[0]["parameters"]["basic.simultaneous_threshold_milliseconds"], 300);
}

#[test]
fn test_chained_operations() {
    let ctx = TestContext::new();
    
    let pkl_content = r#"
module test

import "karabiner.pkl" as karabiner
import "helpers.pkl" as helpers

// Test chaining operations with functional style
simpleConfig: karabiner.SimpleConfig = new {
  complex_modifications = new karabiner.ComplexModifications {
    rules = List(
      new karabiner.Rule {
        description = "Chained Operations"
        manipulators = helpers.charRange("a", "d")
          .map((letter) -> letter.toUpperCase())  // Convert to uppercase
          .filter((letter) -> letter != "C")       // Skip C
          .map((letter) ->                         // Create manipulators
            new karabiner.Manipulator {
              type = "basic"
              from = new karabiner.FromEvent { 
                key_code = letter.toLowerCase()
                modifiers = new karabiner.Modifiers {
                  mandatory = List("left_control")
                }
              }
              to = List(new karabiner.ToEvent { 
                key_code = letter.toLowerCase()
                modifiers = List("left_command")
              })
            }
          )
      }
    )
  }
}

config: karabiner.Config = simpleConfig.toConfig()
"#;
    
    let pkl_file = ctx.write_pkl_file("chained_operations_test.pkl", pkl_content);
    let result = ctx.compile_pkl_to_json(&pkl_file).expect("Failed to compile");
    
    let manipulators = result["config"]["profiles"][0]["complex_modifications"]["rules"][0]["manipulators"].as_array().unwrap();
    // Should have 3 manipulators (a, b, d - skipping c)
    assert_eq!(manipulators.len(), 3);
    assert_eq!(manipulators[0]["from"]["key_code"], "a");
    assert_eq!(manipulators[1]["from"]["key_code"], "b");
    assert_eq!(manipulators[2]["from"]["key_code"], "d");
}