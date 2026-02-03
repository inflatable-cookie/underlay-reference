-- Acme baseline: schema definitions.
--
-- Creates required extensions and logical schemas.
--
-- IMPORTANT:
-- - Do not use `SET search_path` in migrations; always fully qualify objects.

-- Required for gen_random_uuid()
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Underlay schemas
CREATE SCHEMA IF NOT EXISTS auth;
CREATE SCHEMA IF NOT EXISTS account;
CREATE SCHEMA IF NOT EXISTS platform;
CREATE SCHEMA IF NOT EXISTS media;

-- App schema
CREATE SCHEMA IF NOT EXISTS acme;
