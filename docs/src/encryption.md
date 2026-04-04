# Encryption

EchoAccess provides two layers of encryption for sensitive configuration files.

## File-Level Encryption (age)

Entire files (like SSH private keys) are encrypted using the `age` encryption format with a passphrase:

```
Original file → age encrypt (passphrase) → Encrypted blob (stored in cloud)
```

The passphrase derives a master key via argon2 KDF.

## Field-Level Encryption (AES-256-GCM)

For structured config files (TOML, YAML, JSON), individual fields can be encrypted while keeping the file structure readable:

```toml
[database]
host = "db.example.com"                    # plaintext
password = "ENC[AES256-GCM:base64data]"    # encrypted field
```

Field encryption uses the field's path (e.g., `database.password`) as Additional Authenticated Data (AAD), preventing field-swap attacks.

## Session Management

Access to encrypted files requires an unlocked session:

```bash
echo_access unlock    # Unlock with master password
echo_access lock      # Lock session (clears key from memory)
```

The session auto-locks after the configured timeout (default: 15 minutes).

## Security Properties

| Property | Implementation |
|----------|---------------|
| Key derivation | argon2id (memory-hard) |
| File encryption | age passphrase mode |
| Field encryption | AES-256-GCM with AAD |
| Key storage | Memory-only (never written to disk) |
| Session timeout | Configurable auto-lock |
