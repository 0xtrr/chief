# JSON file as the datasource

To use a JSON file as datasource, we first need to ensure that the configuration is correct.
Ensure that `datasource_mode` is set to `Json` and that `json.file_path` is set to `/etc/chief/data.json` 
(or wherever you want to store the data). It doesn't matter what the database configs are because they will be ignored
in this setup and the kinds does not impact the datasource_mode.

```toml
datasource_mode = "Json" # Json or Db (requires a Postgresql database)

[filters.pubkey]
enabled = true # enable or disable public key filter
filter_mode = "Whitelist" # Whitelist or Blacklist

[filters.kind]
enabled = true # enable or disable kind filter
filter_mode = "Blacklist" # Whitelist or Blacklist

[filters.rate_limit]
enabled = false # enable or disable rate limiting feature
max_events = 10 # maximum number of events in the timeframe specified below
time_window = 60 # timeframe for maximum events (in seconds)

[filters.content]
enabled = false # enable or disable content filtering
validated_kinds = [1] # choose which event kinds you want to validate the content field for

[database]
host = ""
port = ""
user = ""
password = ""
dbname = ""

[json]
file_path = "/etc/chief/data.json" # path to json file containing data to filter with
```

Next, copy the `example-data.json` file to the configuration folder like so:

```bash
sudo cp docs/examples/example-data.json /etc/chief/data.json
```

Now just add/remove public keys, kinds and words/sentences from the lists in this file to whitelist/blacklist anything.
