# JSON file as the datasource

To use a JSON file as datasource, we first need to ensure that the configuration is correct.
Ensure that `datasource_mode` is set to `JSON` and that `json.file_path` is set to `/etc/chief/data.json` 
(or wherever you want to store the data). It doesn't matter what the database configs are because they will be ignored
in this setup and the kinds does not impact the datasource_mode.

```toml
datasource_mode = "JSON"

[filters]
public_key = true
public_key_filter_mode = "Whitelist"
kind = true
kind_filter_mode = "Blacklist"
word = false

[database]
host = ""
port = ""
user = ""
password = ""
dbname = ""

[json]
file_path = "/etc/chief/data.json"
```

Next, copy the `example-data.json` file to the configuration folder like so:

```bash
sudo cp docs/examples/example-data.json /etc/chief/data.json
```

Now just add/remove public keys, kinds and words/sentences from the lists in this file to whitelist/blacklist anything.
