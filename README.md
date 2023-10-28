# Chief

Chief is a write policy plugin for Strfry which is a nostr relay software.

## Setup

You'll need to set up a postgresql database somewhere to be able to run this.

### Database

1. Log into the postgresql databsae as the root user and execute the scripts in the `contrib/db` folder.
2. Add anything you want to the blacklisted\_\* tables.

### The plugin

1. Run `cargo build --release`.
2. `sudo cp /target/release/chief /usr/local/bin`.
3. `sudo cp example-config.toml /etc/chief.toml`. This path is currently hardcoded and cannot be changed.
4. Edit the database properties in the chief.toml file.
5. Set the path of the chief executable in the stryfry config (writePolicy.plugin) to `/usr/local/bin/chief`.
