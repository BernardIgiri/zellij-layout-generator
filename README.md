# Zellij Layout Generator

Zellij Layout Generator is a CLI tool for generating custom Zellij layout files from reusable templates. It streamlines the creation of development workflows by combining configuration and template logic into a seamless process. This tool is specific to my personal workflow, but it might be useful for others as well.

## Features

- **Dynamic Layout Generation**: Define reusable templates with placeholders for specific panels and watch commands.
- **Multi-Project Support**: Generate layouts for multiple projects in a single configuration file.
- **Customizable Commands**: Configure individual watch commands for each layout, including the ability to broadcast commands across panes.
- **Broadcast watch commands**: Add the `broadcast` flag to any watch command to automatically enable `script -f` broadcasting for that pane. This will allow you to duplicate the output of shell commands to other terminals.
   - Example:
     ```
    { name = "Dev Server", command = ["npm", "run", "dev"], broadcast = true }
     ```

## Example

### Configuration File (TOML)

Define your Zellij layouts in a `config.toml` file:

```toml
template = "layout_template.kdl"

[[layout]]
path = "projectA_layout.kdl"
watch = [
    { name = "Watcher", command = ["npm", "run", "watch"] },
    { name = "Dev Server", command = ["npm", "run", "dev"], broadcast = true },
    { name = "Logs", command = ["tail", "-f", "logs/dev.log"] }
]

[[layout]]
path = "projectB_layout.kdl"
watch = [
    { name = "Start Server", command = ["yarn", "start"] },
    { name = "Build", command = ["yarn", "build"] },
]
```

- **Key Fields**:
  - `template`: Path to the reusable template file.
  - `layout`: Define one or more layouts.
    - `path`: Output file for the layout.
    - `watch`: List of watch commands to include in the layout.
      - `name`: Pane name for the command.
      - `command`: Command to run in the pane, specified as an array of strings (e.g., `["npm", "run", "watch"]`).
      - `broadcast` (optional): If true, broadcasts input to all panes in this watch group.

### **Template File (KDL)**

Create a reusable template with placeholders for dynamic content:

```kdl
layout {
    tab name="Software Development" {
        pane split_direction="vertical" {
            pane size="20%" {
                pane name="Editor" edit="."
            }
            pane size="80%" split_direction="horizontal" {
                pane size="50%" {
                    pane name="Terminal"
                }
                pane size="50%" name="Watch Panel" stacked=true {
                    ${WATCH_PANELS}
                }
            }
        }
    }
}
```

- **Placeholder**:
  - `${WATCH_PANELS}`: Automatically replaced with panes defined in the `watch` list of your configuration file.

## Generated Layout

Given the example configuration and template, the tool generates output files like `projectA_layout.kdl`:

```kdl
layout {
    tab name="Software Development" {
        pane split_direction="vertical" {
            pane size="20%" {
                pane name="Editor" edit="."
            }
            pane size="80%" split_direction="horizontal" {
                pane size="50%" {
                    pane name="Terminal"
                }
                pane size="50%" name="Watch Panel" stacked=true {
                    pane name="Watcher" command="npm run watch"
                    pane name="Dev Server" command="script" {
                        args "-fec" "npm run dev" ".broadcast"
                    }
                    pane name="Logs" command="tail -f logs/dev.log"
                }
            }
        }
    }
}
```

## Usage

Run the tool with a configuration file:

```bash
zellij-layout-gen config.toml
```

- This reads `config.toml` and generates layout files defined in the `path` field of each layout.

## Installation

Install the tool with `cargo`:

```bash
git clone https://github.com/BernardIgiri/zellij-layout-generator.git
cd zellij-layout-generator
cargo install --path .
```
