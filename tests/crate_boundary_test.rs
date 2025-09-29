// Crate Boundary Enforcement Tests
// Ensures proper separation between domain logic and presentation logic

#[cfg(test)]
mod crate_boundary_tests {
    use std::path::Path;

    #[test]
    fn test_user_crate_has_no_web_dependencies() {
        // Verify that the user crate doesn't depend on web-specific crates
        let user_cargo_toml = std::fs::read_to_string("crates/imkitchen-user/Cargo.toml")
            .expect("Failed to read user Cargo.toml");
        
        // User crate should NOT depend on web frameworks
        assert!(!user_cargo_toml.contains("axum"), "User crate should not depend on axum");
        assert!(!user_cargo_toml.contains("askama"), "User crate should not depend on askama");
        assert!(!user_cargo_toml.contains("tower"), "User crate should not depend on tower-http");
        assert!(!user_cargo_toml.contains("imkitchen-web"), "User crate should not depend on web crate");
        
        // User crate SHOULD only depend on shared types and external libraries
        assert!(user_cargo_toml.contains("imkitchen-shared"), "User crate should depend on shared types");
        assert!(user_cargo_toml.contains("validator"), "User crate should have validation");
        assert!(user_cargo_toml.contains("sqlx"), "User crate should have database access");
    }

    #[test]
    fn test_web_crate_can_depend_on_user_crate() {
        // Verify that the web crate properly depends on user domain
        let web_cargo_toml = std::fs::read_to_string("crates/imkitchen-web/Cargo.toml")
            .expect("Failed to read web Cargo.toml");
        
        // Web crate SHOULD depend on user and shared crates
        assert!(web_cargo_toml.contains("imkitchen-user"), "Web crate should depend on user crate");
        assert!(web_cargo_toml.contains("imkitchen-shared"), "Web crate should depend on shared types");
        
        // Web crate SHOULD have web-specific dependencies
        assert!(web_cargo_toml.contains("axum"), "Web crate should use axum framework");
        assert!(web_cargo_toml.contains("askama"), "Web crate should use askama templates");
        assert!(web_cargo_toml.contains("tower-http"), "Web crate should use tower-http");
    }

    #[test]
    fn test_shared_crate_has_minimal_dependencies() {
        // Verify that shared crate only has necessary dependencies
        let shared_cargo_toml = std::fs::read_to_string("crates/imkitchen-shared/Cargo.toml")
            .expect("Failed to read shared Cargo.toml");
        
        // Shared crate should NOT depend on other internal crates
        assert!(!shared_cargo_toml.contains("imkitchen-user"), "Shared crate should not depend on user crate");
        assert!(!shared_cargo_toml.contains("imkitchen-web"), "Shared crate should not depend on web crate");
        
        // Shared crate should NOT have framework-specific dependencies
        assert!(!shared_cargo_toml.contains("axum"), "Shared crate should not depend on axum");
        assert!(!shared_cargo_toml.contains("askama"), "Shared crate should not depend on askama");
        
        // Shared crate SHOULD have only basic type dependencies
        assert!(shared_cargo_toml.contains("serde"), "Shared crate should have serialization");
        assert!(shared_cargo_toml.contains("validator"), "Shared crate should have validation");
        assert!(shared_cargo_toml.contains("uuid"), "Shared crate should have UUID support");
    }

    #[test]
    fn test_domain_models_are_in_user_crate() {
        // Verify that domain models exist in the correct crate
        assert!(Path::new("crates/imkitchen-user/src/domain").exists(), 
                "User domain logic should be in user crate");
        assert!(Path::new("crates/imkitchen-user/src/domain/user.rs").exists(), 
                "User aggregate should be in user crate");
        assert!(Path::new("crates/imkitchen-user/src/events").exists(), 
                "User events should be in user crate");
        assert!(Path::new("crates/imkitchen-user/src/commands").exists(), 
                "User commands should be in user crate");
        assert!(Path::new("crates/imkitchen-user/src/queries").exists(), 
                "User queries should be in user crate");
    }

    #[test]
    fn test_presentation_logic_is_in_web_crate() {
        // Verify that presentation logic exists in the correct crate
        assert!(Path::new("crates/imkitchen-web/src/handlers").exists(), 
                "HTTP handlers should be in web crate");
        assert!(Path::new("crates/imkitchen-web/templates").exists(), 
                "HTML templates should be in web crate");
        assert!(Path::new("crates/imkitchen-web/static").exists(), 
                "Static assets should be in web crate");
        
        // Check specific template files
        assert!(Path::new("crates/imkitchen-web/templates/auth/login.html").exists(), 
                "Login template should be in web crate");
        assert!(Path::new("crates/imkitchen-web/templates/dashboard/user.html").exists(), 
                "Dashboard template should be in web crate");
    }

    #[test]
    fn test_shared_types_are_in_shared_crate() {
        // Verify that shared types exist in the correct crate
        assert!(Path::new("crates/imkitchen-shared/src/types").exists(), 
                "Shared types should be in shared crate");
        
        // Verify that value objects are shared
        let types_mod = std::fs::read_to_string("crates/imkitchen-shared/src/types/mod.rs")
            .expect("Failed to read shared types");
        
        assert!(types_mod.contains("pub struct Email"), "Email should be a shared type");
        assert!(types_mod.contains("pub struct FamilySize"), "FamilySize should be a shared type");
        assert!(types_mod.contains("pub enum SkillLevel"), "SkillLevel should be a shared type");
    }

    #[test]
    fn test_no_circular_dependencies() {
        // Test dependency graph for cycles
        // This is a simplified check - in a real project you might use a tool like cargo-dependencies
        
        // User crate dependencies
        let user_deps = std::fs::read_to_string("crates/imkitchen-user/Cargo.toml")
            .expect("Failed to read user Cargo.toml");
        assert!(!user_deps.contains("imkitchen-web"), "No circular dependency: user -> web");
        
        // Shared crate dependencies
        let shared_deps = std::fs::read_to_string("crates/imkitchen-shared/Cargo.toml")
            .expect("Failed to read shared Cargo.toml");
        assert!(!shared_deps.contains("imkitchen-user"), "No circular dependency: shared -> user");
        assert!(!shared_deps.contains("imkitchen-web"), "No circular dependency: shared -> web");
    }

    #[test]
    fn test_proper_dependency_direction() {
        // Verify the dependency direction: CLI -> Web -> User -> Shared
        
        // CLI should depend on Web
        let cli_deps = std::fs::read_to_string("Cargo.toml")
            .expect("Failed to read root Cargo.toml");
        assert!(cli_deps.contains("imkitchen-web"), "CLI should depend on web crate");
        
        // Web should depend on User and Shared
        let web_deps = std::fs::read_to_string("crates/imkitchen-web/Cargo.toml")
            .expect("Failed to read web Cargo.toml");
        assert!(web_deps.contains("imkitchen-user"), "Web should depend on user crate");
        assert!(web_deps.contains("imkitchen-shared"), "Web should depend on shared crate");
        
        // User should depend only on Shared
        let user_deps = std::fs::read_to_string("crates/imkitchen-user/Cargo.toml")
            .expect("Failed to read user Cargo.toml");
        assert!(user_deps.contains("imkitchen-shared"), "User should depend on shared crate");
        assert!(!user_deps.contains("imkitchen-web"), "User should NOT depend on web crate");
    }

    #[test]
    fn test_domain_logic_purity() {
        // Verify that domain logic doesn't import presentation concerns
        
        // Check User domain files for presentation imports
        let user_domain = std::fs::read_to_string("crates/imkitchen-user/src/domain/user.rs")
            .expect("Failed to read user domain");
        
        assert!(!user_domain.contains("axum"), "Domain should not import axum");
        assert!(!user_domain.contains("askama"), "Domain should not import askama");
        assert!(!user_domain.contains("Html"), "Domain should not import HTML types");
        assert!(!user_domain.contains("Response"), "Domain should not import HTTP response types");
        
        // Domain SHOULD import domain-appropriate types
        assert!(user_domain.contains("Email") || user_domain.contains("imkitchen_shared"), 
                "Domain should use shared types");
    }

    #[test]
    fn test_presentation_layer_separation() {
        // Verify that presentation layer properly uses domain layer
        
        // Check auth handlers
        let auth_handlers = std::fs::read_to_string("crates/imkitchen-web/src/handlers/auth.rs")
            .expect("Failed to read auth handlers");
        
        // Presentation SHOULD import domain types
        assert!(auth_handlers.contains("imkitchen_user") || auth_handlers.contains("Email"), 
                "Presentation should use domain types");
        assert!(auth_handlers.contains("DirectLoginService") || auth_handlers.contains("LoginCommand"), 
                "Presentation should use domain services");
        
        // Presentation SHOULD have web-specific imports
        assert!(auth_handlers.contains("axum"), "Presentation should use web framework");
        assert!(auth_handlers.contains("Template") || auth_handlers.contains("askama"), 
                "Presentation should use templating");
    }

    #[test]
    fn test_event_sourcing_isolation() {
        // Verify that event sourcing is contained within domain boundaries
        
        // Events should be in user crate
        assert!(Path::new("crates/imkitchen-user/src/events").exists(), 
                "Events should be in domain crate");
        
        // Check that events don't leak into presentation
        let auth_handlers = std::fs::read_to_string("crates/imkitchen-web/src/handlers/auth.rs")
            .expect("Failed to read auth handlers");
        
        // Presentation layer should use services, not events directly
        assert!(!auth_handlers.contains("UserRegistered"), 
                "Presentation should not directly handle events");
        assert!(!auth_handlers.contains("UserLoggedIn"), 
                "Presentation should not directly handle events");
        
        // But should use domain services that encapsulate event handling
        assert!(auth_handlers.contains("DirectLoginService") || auth_handlers.contains("LoginCommand"), 
                "Presentation should use domain services");
    }
}

// Integration test to verify the architecture works end-to-end
#[cfg(test)]
mod architecture_integration_tests {
    use std::process::Command;

    #[test]
    fn test_all_crates_compile_independently() {
        // Test that each crate can compile on its own (no hidden dependencies)
        
        let crates = vec![
            "crates/imkitchen-shared",
            "crates/imkitchen-user", 
            "crates/imkitchen-web",
        ];
        
        for crate_path in crates {
            let output = Command::new("cargo")
                .args(&["check", "--manifest-path", &format!("{}/Cargo.toml", crate_path)])
                .output()
                .expect("Failed to run cargo check");
            
            assert!(output.status.success(), 
                    "Crate {} should compile independently. Error: {}", 
                    crate_path, 
                    String::from_utf8_lossy(&output.stderr));
        }
    }

    #[test] 
    fn test_dependency_graph_is_acyclic() {
        // Use cargo to verify no circular dependencies
        let output = Command::new("cargo")
            .args(&["tree", "--format", "{p}"])
            .output()
            .expect("Failed to run cargo tree");
        
        assert!(output.status.success(), "Dependency tree should be valid");
        
        let tree_output = String::from_utf8_lossy(&output.stdout);
        
        // Should not contain any circular patterns
        assert!(!tree_output.contains("imkitchen-user -> imkitchen-web"), 
                "Should not have circular dependency user -> web");
        assert!(!tree_output.contains("imkitchen-shared -> imkitchen-user"), 
                "Should not have circular dependency shared -> user");
    }
}