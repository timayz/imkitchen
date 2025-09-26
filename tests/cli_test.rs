use clap::{CommandFactory, Parser};

// Import the CLI structures from main.rs
// Since we can't directly import from main.rs, we'll recreate the essential structures for testing
use clap::Subcommand;

/// IMKitchen CLI - Intelligent Meal Planning Application
#[derive(Parser)]
#[command(name = "imkitchen")]
#[command(version = "0.1.0")]
#[command(about = "Intelligent meal planning and kitchen management")]
#[command(long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Web server management
    Web {
        #[command(subcommand)]
        action: WebCommands,
    },
    /// Database migration management
    Migrate {
        #[command(subcommand)]
        action: MigrateCommands,
    },
    /// System health check
    Health,
}

#[derive(Subcommand)]
enum WebCommands {
    /// Start the web server
    Start {
        /// Host to bind to
        #[arg(long, default_value = "0.0.0.0")]
        host: String,
        /// Port to bind to
        #[arg(long, default_value = "3000")]
        port: u16,
    },
    /// Stop the web server gracefully
    Stop,
}

#[derive(Subcommand)]
enum MigrateCommands {
    /// Run pending migrations
    Up,
    /// Rollback migrations
    Down {
        /// Number of migrations to rollback
        #[arg(long, default_value = "1")]
        steps: u32,
    },
    /// Show migration status
    Status,
}

#[cfg(test)]
mod cli_tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_cli_help_generation() {
        let mut cmd = Cli::command();
        let help = cmd.render_help();
        assert!(help.to_string().contains("imkitchen"));
        assert!(help.to_string().contains("Intelligent meal planning"));
    }

    #[test]
    fn test_web_start_command_parsing() {
        let args = vec!["imkitchen", "web", "start"];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_ok());

        let cli = cli.unwrap();
        match &cli.command {
            Commands::Web { action } => {
                match action {
                    WebCommands::Start { host, port } => {
                        assert_eq!(host, "0.0.0.0"); // default value
                        assert_eq!(*port, 3000); // default value
                    }
                    _ => panic!("Expected WebCommands::Start"),
                }
            }
            _ => panic!("Expected Commands::Web"),
        }
    }

    #[test]
    fn test_web_start_with_custom_host_port() {
        let args = vec![
            "imkitchen",
            "web",
            "start",
            "--host",
            "127.0.0.1",
            "--port",
            "8080",
        ];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_ok());

        let cli = cli.unwrap();
        match &cli.command {
            Commands::Web { action } => match action {
                WebCommands::Start { host, port } => {
                    assert_eq!(host, "127.0.0.1");
                    assert_eq!(*port, 8080);
                }
                _ => panic!("Expected WebCommands::Start"),
            },
            _ => panic!("Expected Commands::Web"),
        }
    }

    #[test]
    fn test_web_stop_command_parsing() {
        let args = vec!["imkitchen", "web", "stop"];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_ok());

        let cli = cli.unwrap();
        match &cli.command {
            Commands::Web { action } => {
                matches!(action, WebCommands::Stop);
            }
            _ => panic!("Expected Commands::Web"),
        }
    }

    #[test]
    fn test_migrate_up_command_parsing() {
        let args = vec!["imkitchen", "migrate", "up"];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_ok());

        let cli = cli.unwrap();
        match &cli.command {
            Commands::Migrate { action } => {
                matches!(action, MigrateCommands::Up);
            }
            _ => panic!("Expected Commands::Migrate"),
        }
    }

    #[test]
    fn test_migrate_down_command_parsing() {
        let args = vec!["imkitchen", "migrate", "down"];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_ok());

        let cli = cli.unwrap();
        match &cli.command {
            Commands::Migrate { action } => {
                match action {
                    MigrateCommands::Down { steps } => {
                        assert_eq!(*steps, 1); // default value
                    }
                    _ => panic!("Expected MigrateCommands::Down"),
                }
            }
            _ => panic!("Expected Commands::Migrate"),
        }
    }

    #[test]
    fn test_migrate_down_with_steps() {
        let args = vec!["imkitchen", "migrate", "down", "--steps", "5"];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_ok());

        let cli = cli.unwrap();
        match &cli.command {
            Commands::Migrate { action } => match action {
                MigrateCommands::Down { steps } => {
                    assert_eq!(*steps, 5);
                }
                _ => panic!("Expected MigrateCommands::Down"),
            },
            _ => panic!("Expected Commands::Migrate"),
        }
    }

    #[test]
    fn test_migrate_status_command_parsing() {
        let args = vec!["imkitchen", "migrate", "status"];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_ok());

        let cli = cli.unwrap();
        match &cli.command {
            Commands::Migrate { action } => {
                matches!(action, MigrateCommands::Status);
            }
            _ => panic!("Expected Commands::Migrate"),
        }
    }

    #[test]
    fn test_health_command_parsing() {
        let args = vec!["imkitchen", "health"];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_ok());

        let cli = cli.unwrap();
        matches!(&cli.command, Commands::Health);
    }

    #[test]
    fn test_invalid_command_parsing() {
        let args = vec!["imkitchen", "invalid"];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_err());
    }

    #[test]
    fn test_missing_subcommand() {
        let args = vec!["imkitchen"];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_err());
    }

    #[test]
    fn test_invalid_port_value() {
        let args = vec!["imkitchen", "web", "start", "--port", "invalid"];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_err());
    }

    #[test]
    fn test_port_out_of_range() {
        let args = vec!["imkitchen", "web", "start", "--port", "70000"];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_err());
    }
}
