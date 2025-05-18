#![warn(clippy::avoid_specialized_and_generic_patterns_in_declarative_macro)]

#[derive(Debug)]
enum SafetyLevel {
    Green,
    Yellow,
    Red,
}
// Different device types that need safety checks
struct PressureSensor {/* ... */}
struct TemperatureSensor {/* ... */}
struct EmergencyValve {
    open: bool,
}

trait SafetyCheckForSpecializationIssue {
    fn verify(&self) -> SafetyLevel;
}
// This macro has a pattern ordering issue possible on refactor
macro_rules! impl_safety_trait_specialization_issue {
    // Generic pattern matches any type - including EmergencyValve
    ($u:ty) => {
        impl SafetyCheckForSpecializationIssue for $u {
            fn verify(&self) -> SafetyLevel {
                SafetyLevel::Green
            }
        }
    };
    // Generic pattern matches any type - including EmergencyValve
    ($t:ty, EmergencyValve) => {
        impl SafetyCheckForSpecializationIssue for $t {
            fn verify(&self) -> SafetyLevel {
                SafetyLevel::Green
            }
        }
    };
    // Special pattern for EmergencyValve - but never gets matched
    (EmergencyValve) => {
        //~^ avoid_specialized_and_generic_patterns_in_declarative_macro
        impl SafetyCheckForSpecializationIssue for EmergencyValve {
            fn verify(&self) -> SafetyLevel {
                // Emergency valve must be open for safety
                if !self.open {
                    SafetyLevel::Red
                } else {
                    SafetyLevel::Green
                }
            }
        }
    };
}
impl_safety_trait_specialization_issue!(EmergencyValve);
impl_safety_trait_specialization_issue!(PressureSensor);
impl_safety_trait_specialization_issue!(TemperatureSensor);

trait SafetyCheckWithoutSpecializationIssue {
    fn verify(&self) -> SafetyLevel;
}
// This macro has a pattern ordering issue possible on refactor
macro_rules! impl_safety_trait_without_specialization_issue {
    // Generic pattern matches any type - including EmergencyValve
    ($t:ty) => {
        impl SafetyCheckWithoutSpecializationIssue for $t {
            fn verify(&self) -> SafetyLevel {
                SafetyLevel::Green
            }
        }
    };
}
impl_safety_trait_without_specialization_issue!(PressureSensor);
impl_safety_trait_without_specialization_issue!(TemperatureSensor);
impl SafetyCheckWithoutSpecializationIssue for EmergencyValve {
    fn verify(&self) -> SafetyLevel {
        // Emergency valve must be open for safety
        if !self.open {
            SafetyLevel::Red
        } else {
            SafetyLevel::Green
        }
    }
}

fn main() {
    // test code goes here
}
