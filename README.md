# Chief

Chief is a write policy plugin for [Strfry](https://github.com/hoytech/strfry) (which is a [nostr](https://github.com/nostr-protocol/nostr) relay software).
It enables relay operators to blacklist or whitelist public keys, event kinds and specific words or sentences using either
a JSON file or a postgresql database.

## Setup

### Compile and configure

1. Compile from source
   - Run `cargo build --release`.
2. Put the compiled binary where you want it to run from
   - E.g. `sudo cp target/release/chief /usr/local/bin`.
3. Create a folder in `/etc` where your configuration files will live and copy the example config to that folder
    1. `sudo mkdir /etc/chief/`
    2. `sudo cp docs/examples/example-config.toml /etc/chief/config.toml`.
        - This path is currently hardcoded and cannot be changed.
4. Configure strfry to use Chief as the write policy
   - Under "relay.writePolicy", set the plugin to `/usr/local/bin/chief`
```
    writePolicy {
        # If non-empty, path to an executable script that implements the writePolicy plugin logic
        plugin = "/usr/local/bin/chief"
    }
```

### Select a datasource
The datasource contains the public keys, kinds and/or words you want to either whitelist or blacklist.
This application supports two different datasources: a JSON file or a postgresql database.

#### JSON

To use a JSON file as the datasource, please read [this document](docs/json_datasource.md).

#### Postgresql database

To use a postgresql database as the datasource, please read [this document](docs/postgresql_datasource.md).

### Filters

The application has three filters: public keys, kinds and content, and each of them can be individually activated or 
deactivated in the configuration file. For public keys and kinds, you can also choose to either blacklist or whitelist
the items in the lists by setting the filter_mode to "Blacklist" or "Whitelist". I don't think it makes any sense to 
whitelist content, so it is always blacklisted.