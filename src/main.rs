mod config;

use clap::Parser;
use config::{Config, Layout};
use shell_quote::{QuoteRefExt, Sh};
use std::{error::Error, fs, path::Path};

const PLACEHOLDER: &str = "${WATCH_PANELS}";

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
    for layout in &project_config.layout {
        let rendered_layout = render_layout(template, layout)?;
        save_layout(layout.path.as_path(), &rendered_layout)?;
    }
    Ok(())
}

fn extract_args(command: &[String], quote: bool) -> Result<(String, String), Box<dyn Error>> {
    if command.is_empty() {
        return Err("Command cannot be empty.".into());
    }
    let executable = String::from_utf8(command[0].quoted(Sh))?;
    let args = command[1..]
        .iter()
        .map(|arg| {
            let arg = arg.quoted(Sh);
            let arg = String::from_utf8(arg)?;
            let arg = if quote { format!("\"{}\"", arg) } else { arg };
            Ok(arg)
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;

    let args = args.join(" ");
    Ok((executable, args))
}

fn render_layout(template: &str, layout: &Layout) -> Result<String, Box<dyn Error>> {
    let watch_panels = layout
        .watch
        .iter()
        .map(|watch| {
            // Adjust command based on the broadcast flag
            let command = watch.command.clone();
            Ok(if watch.broadcast {
                let (executable, args) = extract_args(&command, false)?;
                format!(
                    "pane name=\"{}\" command=\"script\" {{\n    args \"-fec\" \"{} {}\" \".broadcast\"\n}}",
                    watch.name,
                    executable,
                    args
                )
            } else {
                let (executable, args) = extract_args(&command, true)?;

                if args.is_empty() {
                    // No args, render command on the same line
                    format!(
                        "pane name=\"{}\" command=\"{}\"",
                        watch.name,
                        executable
                    )
                } else {
                    // Escape and format arguments safely
                    format!(
                        "pane name=\"{}\" command=\"{}\" {{\n    args {}\n}}",
                        watch.name,
                        executable,
                        args
                    )
                }
            })
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;

    let watch_panels = watch_panels.join("\n");
    if template.contains(PLACEHOLDER) {
        Ok(template.replace(PLACEHOLDER, &watch_panels))
    } else {
        Err("The watch panel placeholder is missing!".into())
    }
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
    use super::*;
    use config::{Layout, Watch};

    fn normalize_whitespace(input: &str) -> String {
        input.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    #[test]
    fn test_parse_config() {
        let config = r#"
        template = "layout_template.kdl"

        [[layout]]
        path = "projectA_layout.kdl"
        watch = [
            { name = "Watcher", command = ["npm", "run", "watch"] },
            { name = "Dev Server", command = ["npm", "run", "dev"] }
        ]
        "#;

        let project_config = parse_config(config).expect("Failed to parse config");
        assert_eq!(
            project_config.template.into_os_string().to_str(),
            "layout_template.kdl".into()
        );
        assert_eq!(project_config.layout.len(), 1);
        assert_eq!(
            project_config.layout[0]
                .path
                .clone()
                .into_os_string()
                .to_str(),
            "projectA_layout.kdl".into()
        );
        assert_eq!(project_config.layout[0].watch.len(), 2);
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
                Watch {
                    name: "Watcher".to_string(),
                    command: vec!["npm".into(), "run".into(), "watch".into()],
                    broadcast: false,
                },
                Watch {
                    name: "Dev Server".to_string(),
                    command: vec![
                        "npm".into(),
                        "run".into(),
                        "dev".into(),
                        "--host".into(),
                        "0.0.0.0".into(),
                    ],
                    broadcast: false,
                },
            ],
        };

        let result = render_layout(template, &layout).expect("layout to render");
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
            layout: vec![Layout {
                path: path.clone(),
                watch: vec![Watch {
                    name: "Test".to_string(),
                    command: vec!["echo".into(), "Hello".into()],
                    broadcast: false,
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

    #[test]
    fn test_render_layout_with_broadcast() {
        let template = r#"
        layout {
            ${WATCH_PANELS}
        }
        "#;

        let layout = Layout {
            path: "test.kdl".parse().expect("valid path"),
            watch: vec![
                Watch {
                    name: "Dev".to_string(),
                    command: vec![
                        "yarn".into(),
                        "dev".into(),
                        "--host".into(),
                        "0.0.0.0".into(),
                    ],
                    broadcast: true,
                },
                Watch {
                    name: "Lint".to_string(),
                    command: vec!["yarn".into(), "watch".into()],
                    broadcast: false,
                },
            ],
        };

        let result = render_layout(template, &layout).expect("layout to render");
        let expected = r#"
        layout {
            pane name="Dev" command="script" {
                args "-fec" "yarn dev --host 0.0.0.0" ".broadcast"
            }
            pane name="Lint" command="yarn" {
                args "watch"
            }
        }
        "#;

        assert_eq!(
            normalize_whitespace(&result),
            normalize_whitespace(expected),
            "Rendered layout with secure broadcast does not match expected output"
        );
    }

    #[test]
    fn test_empty_layouts() {
        let config = r#"
        template = "layout_template.kdl"
        layout = []
        "#;

        let project_config = parse_config(config).expect("Failed to parse config");
        assert_eq!(project_config.layout.len(), 0, "Layouts should be empty");
    }

    #[test]
    fn test_missing_placeholder_in_template() {
        let template = r#"
        layout {
            pane name="Placeholder Missing"
        }
        "#;

        let layout = Layout {
            path: "test.kdl".parse().expect("valid path"),
            watch: vec![Watch {
                name: "Test".to_string(),
                command: vec!["echo".into(), "Hello".into()],
                broadcast: false,
            }],
        };

        let result = render_layout(template, &layout);
        assert!(
            result.is_err(),
            "Rendering should fail due to missing placeholder"
        );
        assert_eq!(
            result.unwrap_err().to_string(),
            "The watch panel placeholder is missing!",
            "Expected error for missing placeholder"
        );
    }

    #[test]
    fn test_commands_with_special_characters() {
        let template = r#"
        layout {
            ${WATCH_PANELS}
        }
        "#;

        let layout = Layout {
            path: "test.kdl".parse().expect("valid path"),
            watch: vec![Watch {
                name: "Special Characters".to_string(),
                command: vec![
                    "echo".into(),
                    "Hello World".into(),
                    "\"Quoted Arg\"".into(),
                    "Semi;Colon".into(),
                ],
                broadcast: false,
            }],
        };

        let result = render_layout(template, &layout).expect("layout to render");
        let expected = r#"
        layout {
            pane name="Special Characters" command="echo" {
                args "Hello' World'" "'"Quoted Arg"'" "Semi';Colon'"
            }
        }
        "#;

        assert_eq!(
            normalize_whitespace(&result),
            normalize_whitespace(expected),
            "Rendered layout does not match expected output for special characters"
        );
    }

    #[test]
    fn test_invalid_toml_config() {
        let invalid_config = r#"
        template = "layout_template.kdl"
        [[layout]]
        path = "projectA_layout.kdl"
        watch = [
            { name = "Watcher", command = "not-an-array" } # Invalid command format
        ]
        "#;

        let result = parse_config(invalid_config);
        assert!(result.is_err(), "Parsing should fail for invalid TOML");
    }

    #[test]
    fn test_broadcast_commands_with_complex_arguments() {
        let template = r#"
        layout {
            ${WATCH_PANELS}
        }
        "#;

        let layout = Layout {
            path: "test.kdl".parse().expect("valid path"),
            watch: vec![Watch {
                name: "Broadcast Test".to_string(),
                command: vec!["echo".into(), "Complex Arguments".into(), "Here".into()],
                broadcast: true,
            }],
        };

        let result = render_layout(template, &layout).expect("layout to render");
        let expected = r#"
        layout {
            pane name="Broadcast Test" command="script" {
                args "-fec" "echo Complex' Arguments' Here" ".broadcast"
            }
        }
        "#;

        assert_eq!(
            normalize_whitespace(&result),
            normalize_whitespace(expected),
            "Broadcast layout does not match expected output for complex arguments"
        );
    }

    #[test]
    fn test_broadcast_commands_with_two_arguments() {
        let template = r#"
        layout {
            ${WATCH_PANELS}
        }
        "#;

        let layout = Layout {
            path: "test.kdl".parse().expect("valid path"),
            watch: vec![Watch {
                name: "Broadcast Test".to_string(),
                command: vec!["yarn".into(), "dev".into()],
                broadcast: true,
            }],
        };

        let result = render_layout(template, &layout).expect("layout to render");
        let expected = r#"
        layout {
            pane name="Broadcast Test" command="script" {
                args "-fec" "yarn dev" ".broadcast"
            }
        }
        "#;

        assert_eq!(
            normalize_whitespace(&result),
            normalize_whitespace(expected),
            "Broadcast layout does not match expected output for complex arguments"
        );
    }
}
