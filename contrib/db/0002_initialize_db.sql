-- Create a new database
CREATE DATABASE chief;

-- Connect to the newly created database
\c chief;

-- Create a table for storing blacklisted public keys
CREATE TABLE blacklisted_pubkeys
(
    id     SERIAL PRIMARY KEY,
    pubkey TEXT NOT NULL
);

-- Create a table for storing blacklisted kinds
CREATE TABLE blacklisted_kinds
(
    id   SERIAL PRIMARY KEY,
    kind INTEGER NOT NULL
);

-- Create a table for storing blacklisted words
CREATE TABLE blacklisted_words
(
    id   SERIAL PRIMARY KEY,
    word TEXT NOT NULL
);

ALTER TABLE blacklisted_pubkeys OWNER TO chief;
ALTER TABLE blacklisted_kinds OWNER TO chief;
ALTER TABLE blacklisted_words OWNER TO chief;


-- Grant necessary privileges to the chief user
--GRANT CONNECT ON DATABASE chief TO chief;
--GRANT USAGE ON SCHEMA public TO chief;
--GRANT SELECT, INSERT, DELETE, UPDATE ON TABLE blacklisted_pubkeys TO chief;
--GRANT SELECT, INSERT, DELETE, UPDATE ON TABLE blacklisted_kinds TO chief;
--GRANT USAGE, SELECT, UPDATE ON SEQUENCE blacklisted_kinds_id_seq TO chief;
--GRANT SELECT, INSERT, DELETE, UPDATE ON TABLE blacklisted_words TO chief;

-- Create an index on the 'word' column for efficient queries
CREATE INDEX idx_blacklisted_words_word ON blacklisted_words(word);
-- Create an index on the 'pubkey' column for efficient queries
CREATE INDEX idx_blacklisted_pubkeys_pubkey ON blacklisted_pubkeys(pubkey);
-- Create an index on the 'kinds' column for efficient queries
CREATE INDEX idx_blacklisted_kinds_kind ON blacklisted_kinds(kind);
