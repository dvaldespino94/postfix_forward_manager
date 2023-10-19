# Email Forward Manager

> This is a simple utility to manage postfix virtual user lists on remote servers, using a simple configuration file:

```toml
username = "the_user_name"
servers = [
    { addr = "127.0.0.1", port = 22, config_path = "/etc/postfix/virtual" },
    { addr = "127.0.0.1", port = 22, config_path = "/etc/postfix/virtualuser" },
    # ... there is no limit in how many servers you can add
]
```

> The only requirement is that the file(named config.toml) is located in the cwd.