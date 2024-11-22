# Zellij Template Generator

**Example:**

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

**Usage:**

`zellij-layout-gen config.toml`

**Install:**

`cargo install --path .`
