# Zellij Template Generator

**Example**

**Config:**

```toml
template = "layout_template.kdl"

[[layout]]
path = "projectA_layout.kdl"
watch = [
    { name = "Watcher", command = "npm run watch" },
    { name = "Dev Server", command = "npm run dev" },
    { name = "Logs", command = "tail -f logs/dev.log" }
]

[[layout]]
path = "projectB_layout.kdl"
watch = [
    { name = "Start Server", command = "yarn start" },
    { name = "Build", command = "yarn build" },
]
```

**Template:**
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

**Usage:**

`zellij-layout-gen config.toml`

**Install:**

`cargo install --path .`
