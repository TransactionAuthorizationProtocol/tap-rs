-- Create customers table for storing party information
CREATE TABLE IF NOT EXISTS customers (
    id TEXT PRIMARY KEY,
    agent_did TEXT NOT NULL,
    
    -- Schema type (Person, Organization, Thing)
    schema_type TEXT NOT NULL CHECK (schema_type IN ('Person', 'Organization', 'Thing')),
    
    -- Core fields for natural persons
    given_name TEXT,
    family_name TEXT,
    display_name TEXT,
    
    -- Core fields for organizations
    legal_name TEXT,
    lei_code TEXT,
    mcc_code TEXT,
    
    -- Address fields (common to both)
    address_country TEXT,
    address_locality TEXT,
    postal_code TEXT,
    street_address TEXT,
    
    -- Full schema.org JSON-LD profile
    profile TEXT NOT NULL, -- JSONB storing complete schema.org data
    
    -- Cached IVMS101 data for Travel Rule compliance
    ivms101_data TEXT, -- JSONB storing IVMS101 formatted data
    
    -- Metadata
    verified_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create customer_identifiers table for storing multiple identifiers per customer
CREATE TABLE IF NOT EXISTS customer_identifiers (
    id TEXT PRIMARY KEY, -- The IRI itself (did:example:123, mailto:user@example.com, etc.)
    customer_id TEXT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    identifier_type TEXT NOT NULL CHECK (identifier_type IN ('did', 'email', 'phone', 'url', 'account', 'other')),
    verified BOOLEAN NOT NULL DEFAULT FALSE,
    verification_method TEXT, -- How it was verified (signature, email confirmation, etc.)
    verified_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(id, customer_id) -- Same identifier can't be linked to same customer twice
);

-- Create customer_relationships table for tracking relationships between entities
CREATE TABLE IF NOT EXISTS customer_relationships (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    relationship_type TEXT NOT NULL, -- acts_for, controls, manages, etc.
    related_identifier TEXT NOT NULL, -- IRI of the related entity
    proof TEXT, -- JSONB storing any cryptographic proofs or confirmations
    confirmed_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(customer_id, relationship_type, related_identifier)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_customers_agent_did ON customers(agent_did);
CREATE INDEX IF NOT EXISTS idx_customers_schema_type ON customers(schema_type);
CREATE INDEX IF NOT EXISTS idx_customers_given_name ON customers(given_name);
CREATE INDEX IF NOT EXISTS idx_customers_family_name ON customers(family_name);
CREATE INDEX IF NOT EXISTS idx_customers_legal_name ON customers(legal_name);
CREATE INDEX IF NOT EXISTS idx_customers_lei_code ON customers(lei_code);
CREATE INDEX IF NOT EXISTS idx_customers_address_country ON customers(address_country);

CREATE INDEX IF NOT EXISTS idx_customer_identifiers_customer_id ON customer_identifiers(customer_id);
CREATE INDEX IF NOT EXISTS idx_customer_identifiers_type ON customer_identifiers(identifier_type);
CREATE INDEX IF NOT EXISTS idx_customer_identifiers_verified ON customer_identifiers(verified);

CREATE INDEX IF NOT EXISTS idx_customer_relationships_customer_id ON customer_relationships(customer_id);
CREATE INDEX IF NOT EXISTS idx_customer_relationships_type ON customer_relationships(relationship_type);
CREATE INDEX IF NOT EXISTS idx_customer_relationships_related ON customer_relationships(related_identifier);

-- Trigger to update the updated_at timestamp (SQLite syntax)
CREATE TRIGGER IF NOT EXISTS update_customers_updated_at
    AFTER UPDATE ON customers
    FOR EACH ROW
BEGIN
    UPDATE customers SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;