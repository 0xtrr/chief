# Postgresql as the datasource

To use a Postgresql database as the datasource, you'll first need to set up postgresql on your system.
Digital Ocean has a bunch of easy to follow guides for this, e.g. [this guide](https://www.digitalocean.com/community/tutorials/how-to-install-and-use-postgresql-on-ubuntu-22-04).

## Database setup
Log into the postgresql database as the root user and execute the scripts in the [contrib/db](contrib/db) folder, and you should be good to go.

## Chief configuration file
Now we need to ensure that the config file has the correct details. Here is an example of a config file that uses a postgresql database as the datasource.
```toml
datasource_mode = "DB"

[filters]
public_key = true
public_key_filter_mode = "Whitelist"
kind = true
kind_filter_mode = "Blacklist"
word = false

[database]
host = "localhost"
port = "5432"
user = "chief"
password = "changeme"
dbname = "chief"

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
