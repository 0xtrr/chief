# Chief

Chief is a write policy plugin for Strfry which is a nostr relay software.
It enables blacklisting of public keys, kind numbers and specific words and sentences.

## Setup

You'll need to set up a postgresql database somewhere to be able to run this.

### Database

1. Log into the postgresql databsae as the root user and execute the scripts in the `contrib/db` folder.

### The plugin

1. Run `cargo build --release`.
2. `sudo cp /target/release/chief /usr/local/bin`.
3. `sudo cp example-config.toml /etc/chief.toml`. This path is currently hardcoded and cannot be changed.
4. Edit the database properties in the chief.toml file.
5. Set the path of the chief executable in the stryfry config (writePolicy.plugin) to `/usr/local/bin/chief`.

## Usage

When you want to blacklist something, you'll have to insert a new row into the database. Here are some examples for each of the database tables in use.

### Blacklist a public key

Remember that this must be a hex key and not a npub.

```sql
INSERT INTO blacklisted_pubkey (pubkey) VALUES ('54a62b4309734f4ea2bff150307af9ff55196988270b5df8a85701503a9802e3');
```

### Blacklist a word

```sql
INSERT INTO blacklisted_words (word) VALUES ('twitter');
```

### Blacklist a kind

```sql
INSERT INTO blacklisted_kind (kind) VALUES (1064);
```
