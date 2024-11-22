mod config;

use clap::Parser;
use config::{Config, Layout};
use std::{error::Error, fs, path::Path};

/// A CLI tool to generate Zellij layouts from templates
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the configuration file
    #[arg(short, long)]
    config: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let config_contents = fs::read_to_string(&cli.config)?;

    let project_config = parse_config(&config_contents)?;
    let template_contents = fs::read_to_string(&project_config.template)?;

    generate_layouts(&project_config, &template_contents)?;

    println!("All layouts have been generated.");
    Ok(())
}

fn parse_config(config: &str) -> Result<Config, Box<dyn Error>> {
    let project_config: Config = toml::from_str(config)?;
    Ok(project_config)
}

fn generate_layouts(project_config: &Config, template: &str) -> Result<(), Box<dyn Error>> {
    for layout in &project_config.layouts {
        let rendered_layout = render_layout(template, layout);
        save_layout(layout.path.as_path(), &rendered_layout)?;
    }
    Ok(())
}

fn render_layout(template: &str, layout: &Layout) -> String {
    let watch_panels = layout
        .watch
        .iter()
        .map(|watch_command| {
            // Split the command into executable and args (if args are present)
            let parts: Vec<&str> = watch_command.command.split_whitespace().collect();
            let executable = parts[0];
            let args = &parts[1..];

            if args.is_empty() {
                // No args, render command on the same line
                format!(
                    "pane name=\"{}\" command=\"{}\"",
                    watch_command.name, executable
                )
            } else {
                // With args, use child braces to include args
                let args_formatted = args
                    .iter()
                    .map(|arg| format!("\"{}\"", arg)) // Quote each argument
                    .collect::<Vec<_>>()
                    .join(" ");
                format!(
                    "pane name=\"{}\" command=\"{}\" {{\n    args {}\n}}",
                    watch_command.name, executable, args_formatted
                )
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    template.replace("${WATCH_PANELS}", &watch_panels)
}

fn save_layout(path: &Path, content: &str) -> Result<(), Box<dyn Error>> {
    let output_file_path = std::path::Path::new(path);

    if let Some(parent) = output_file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use config::{Layout, WatchCommand};

    fn normalize_whitespace(input: &str) -> String {
        input.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    #[test]
    fn test_parse_config() {
        let config = r#"
        template = "layout_template.kdl"

        [[layouts]]
        path = "projectA_layout.kdl"
        watch = [
            { name = "Watcher", command = "npm run watch" },
            { name = "Dev Server", command = "npm run dev" }
        ]
        "#;

        let project_config = parse_config(config).expect("Failed to parse config");
        assert_eq!(
            project_config.template,
            "layout_template.kdl"
                .parse::<PathBuf>()
                .expect("valid path")
        );
        assert_eq!(project_config.layouts.len(), 1);
        assert_eq!(project_config.layouts[0].watch.len(), 2);
    }

    #[test]
    fn test_render_layout() {
        let template = r#"
        layout {
            ${WATCH_PANELS}
        }
        "#;

        let layout = Layout {
            path: "test.kdl".parse().expect("valid path"),
            watch: vec![
                WatchCommand {
                    name: "Watcher".to_string(),
                    command: "npm run watch".to_string(),
                },
                WatchCommand {
                    name: "Dev Server".to_string(),
                    command: "npm run dev --host 0.0.0.0".to_string(),
                },
            ],
        };

        let result = render_layout(template, &layout);
        let expected = r#"
        layout {
            pane name="Watcher" command="npm" {
                args "run" "watch"
            }
            pane name="Dev Server" command="npm" {
                args "run" "dev" "--host" "0.0.0.0"
            }
        }
        "#;

        assert_eq!(
            normalize_whitespace(&result),
            normalize_whitespace(expected),
            "Rendered layout does not match expected output"
        );
    }

    #[test]
    fn test_generate_layouts() {
        use tempfile::NamedTempFile;
        let temp_output = NamedTempFile::new().expect("Failed to create temporary file");
        let path = temp_output.path().to_path_buf();

        let project_config = Config {
            template: "template.kdl".parse().expect("valid path"),
            layouts: vec![Layout {
                path: path.clone(),
                watch: vec![WatchCommand {
                    name: "Test".to_string(),
                    command: "echo Hello".to_string(),
                }],
            }],
        };

        let template = "layout { ${WATCH_PANELS} }";

        generate_layouts(&project_config, template).expect("Failed to generate layouts");

        let generated_content = fs::read_to_string(path).expect("Failed to read output");
        let expected_pane = r#"pane name="Test" command="echo" {
            args "Hello"
        }"#;

        assert!(
            normalize_whitespace(&generated_content).contains(&normalize_whitespace(expected_pane)),
            "Generated content does not match expected output"
        );
    }
}
