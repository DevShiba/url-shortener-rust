-- Keyspace: SimpleStrategy for local dev.
-- In production, use NetworkTopologyStrategy with a replication_factor per datacenter.
CREATE KEYSPACE IF NOT EXISTS url_shortener
    WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1};

-- Main table.
-- Partition key = shortcode → every lookup is a single-partition read (O(1)).
-- No secondary indexes needed — reads and writes are always by shortcode.
CREATE TABLE IF NOT EXISTS url_shortener.url (
    shortcode  TEXT PRIMARY KEY,
    long_url   TEXT,
    created_at TIMESTAMP
);
