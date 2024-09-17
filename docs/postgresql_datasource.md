# Postgresql as the datasource

To use a Postgresql database as the datasource, you'll first need to set up postgresql on your system.
Digital Ocean has a bunch of easy to follow guides for this, e.g. [this guide](https://www.digitalocean.com/community/tutorials/how-to-install-and-use-postgresql-on-ubuntu-22-04).

## Database setup
Log into the postgresql database as the root user and execute the scripts in the [contrib/db](../contrib/db) folder, and you should be good to go.

## Chief configuration file
Now we need to ensure that the config file has the correct details. Here is an example of a config file that uses a postgresql database as the datasource.
```toml
datasource_mode = "Db"

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
host = "localhost" # postgresql database url
port = "5432" # postgresql database port
user = "chief" # database user
password = "changeme" # database password
dbname = "chief" # database table name

[json]
file_path = ""
```

## Managing the database

When you want to blacklist/whitelist something, you'll have to insert a new row into the database.
Here are some examples for each of the database tables in use.

### Add/remove a public key

Remember that this must be a hex key and not a Bech32 encoded key (npub).
```sql
INSERT INTO public_keys(publickey) VALUES ('54a62b4309734f4ea2bff150307af9ff55196988270b5df8a85701503a9802e3');
```
```sql
DELETE FROM public_keys WHERE publickey = '54a62b4309734f4ea2bff150307af9ff55196988270b5df8a85701503a9802e3';
```

### Add/remove a word

```sql
INSERT INTO words(word) VALUES ('twitter');
```
```sql
DELETE FROM words WHERE word = 'twitter';
```

### Add/remove a kind

```sql
INSERT INTO kinds(kind) VALUES (1064);
```
```sql
DELETE FROM kinds WHERE kind = 1064;
```
