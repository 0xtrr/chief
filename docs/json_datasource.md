# JSON file as the datasource

To use a JSON file as datasource, we first need to ensure that the configuration file looks like this:
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
sudo cp docs/examples/example-data.json /etc/chief
```

Now just add/remove public keys, kinds and words/sentences from the lists in this file to whitelist/blacklist anything.
