---
title: Vault Format
description: The .emberkeys vault file format specification.
order: 4
---

# .emberkeys Vault Format

The vault is a single encrypted file with the `.emberkeys` extension. The format is open, versioned, and fully documented. Third-party implementations are encouraged.

## File Structure

```
[4 bytes]   Magic: 0x454D424B ("EMBK")
[2 bytes]   Format version (currently 0x0001)
[16 bytes]  Argon2id salt
[12 bytes]  AES-256-GCM nonce
[4 bytes]   Encrypted payload length
[N bytes]   Encrypted payload (JSON)
[16 bytes]  AES-256-GCM authentication tag
```

Total header size: 38 bytes.

## Decrypted Payload

The encrypted payload is a JSON object:

```json
{
  "version": 1,
  "created": "2025-01-01T00:00:00Z",
  "modified": "2025-01-01T00:00:00Z",
  "entries": [
    {
      "id": "uuid-v4",
      "entry_type": "login",
      "title": "Example",
      "url": "https://example.com",
      "username": "user@example.com",
      "password": "...",
      "totp_secret": "...",
      "notes": "...",
      "tags": ["work"],
      "favourite": false,
      "custom_fields": [
        {
          "name": "API Key",
          "field_type": "hidden",
          "value": "secret123"
        }
      ],
      "card": null,
      "identity": null,
      "password_history": [
        {
          "password": "old_password",
          "changed_at": "2025-01-01T00:00:00Z"
        }
      ],
      "created": "2025-01-01T00:00:00Z",
      "modified": "2025-01-01T00:00:00Z"
    }
  ]
}
```

## Entry Types

### Login

Standard login credentials.

```json
{
  "entry_type": "login",
  "title": "GitHub",
  "url": "https://github.com",
  "username": "user@example.com",
  "password": "secret123",
  "totp_secret": "JBSWY3DPEHPK3PXP"
}
```

### Note

Secure note for freeform text.

```json
{
  "entry_type": "note",
  "title": "WiFi Password",
  "notes": "Network: MyWiFi\nPassword: secret123"
}
```

### Card

Payment card details.

```json
{
  "entry_type": "card",
  "title": "Visa",
  "card": {
    "cardholder": "John Doe",
    "number": "4111111111111111",
    "expiry": "12/25",
    "cvv": "123",
    "pin": "1234"
  }
}
```

### Identity

Personal identity information.

```json
{
  "entry_type": "identity",
  "title": "Personal",
  "identity": {
    "first_name": "John",
    "middle_name": "M",
    "last_name": "Doe",
    "email": "john@example.com",
    "phone": "+1-555-0123",
    "address_line1": "123 Main St",
    "address_line2": "Apt 4",
    "city": "Springfield",
    "state": "IL",
    "postal_code": "62701",
    "country": "US"
  }
}
```

### Authenticator

Standalone 2FA (TOTP) codes.

```json
{
  "entry_type": "authenticator",
  "title": "Google Backup Code",
  "totp_secret": "JBSWY3DPEHPK3PXP"
}
```

## Custom Fields

Entries can have custom fields with these types:

| Type | Description |
| --- | --- |
| `text` | Plain text |
| `hidden` | Masked in UI (like passwords) |
| `url` | URL link |
| `date` | Date value |

## Encryption

- **Cipher:** AES-256-GCM (authenticated encryption)
- **Key derivation:** Argon2id (memory: 64MB, iterations: 3, parallelism: 4)
- **Salt:** 16 bytes, randomly generated per vault
- **Nonce:** 12 bytes, randomly generated per save operation
- **Master password:** never stored, never written to disk, zeroed from memory on lock
