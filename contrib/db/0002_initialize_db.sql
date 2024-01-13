-- Create a new database
CREATE DATABASE chief;

-- Connect to the newly created database
\c chief;

-- Create a table for storing public keys
CREATE TABLE public_keys
(
    id     SERIAL PRIMARY KEY,
    publickey TEXT NOT NULL
);

-- Create a table for storing kinds
CREATE TABLE kinds
(
    id   SERIAL PRIMARY KEY,
    kind INTEGER NOT NULL
);

-- Create a table for storing blacklisted words
CREATE TABLE words
(
    id   SERIAL PRIMARY KEY,
    word TEXT NOT NULL
);

ALTER TABLE public_keys OWNER TO chief;
ALTER TABLE kinds OWNER TO chief;
ALTER TABLE words OWNER TO chief;

-- Create an index on the 'public_keys' column for efficient queries
CREATE INDEX idx_publickeys_publickey ON public_keys(publickey);
-- Create an index on the 'word' column for efficient queries
CREATE INDEX idx_words_word ON words(word);
-- Create an index on the 'kinds' column for efficient queries
CREATE INDEX idx_kinds_kind ON kinds(kind);
