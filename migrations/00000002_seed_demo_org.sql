INSERT INTO organisation (id, name, default_auth)
VALUES ('demo-org', 'Demo Organisation', '{}')
ON CONFLICT (id) DO NOTHING;
