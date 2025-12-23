# git-credential-guts

Git credential helper for seamless Guts authentication.

## Installation

### From Source

```bash
cd tools/git-credential-guts
cargo install --path .
```

### Configure Git

```bash
# Global configuration (all Guts hosts)
git config --global credential.helper guts

# Or for a specific host
git config --global credential.https://guts.network.helper guts
```

## Usage

### Interactive Setup

```bash
git-credential-guts configure
# Enter your personal access token when prompted
```

### With Token

```bash
git-credential-guts configure --token guts_xxx
```

### Custom Host

```bash
git-credential-guts configure --token guts_xxx --host api.guts.network
```

### List Configured Hosts

```bash
git-credential-guts list
```

### Remove Configuration

```bash
git-credential-guts remove guts.network
```

## How It Works

The credential helper stores your Guts personal access token securely in the system keyring (macOS Keychain, Windows Credential Manager, or Linux Secret Service).

When Git needs credentials for a Guts host:

1. Git calls `git-credential-guts get`
2. The helper checks if the host is a Guts host
3. If yes, it retrieves the token from the secure keyring
4. Returns the credentials to Git

## Token Format

Guts personal access tokens follow this format:

```
guts_<prefix>_<secret>
```

For example: `guts_abc12345_XXXXXXXXXXXXXXXXXXXXXXXX`

## Security

- Tokens are stored in the system keyring, not in plain text
- The helper only responds to Guts hosts (*.guts.network, localhost)
- Tokens are never logged or displayed in full

## Troubleshooting

### Credentials Not Working

1. Verify your token is valid:
   ```bash
   curl -H "Authorization: Bearer guts_xxx" https://api.guts.network/api/user
   ```

2. Reconfigure the helper:
   ```bash
   git-credential-guts remove guts.network
   git-credential-guts configure
   ```

### Keyring Not Available

On headless Linux systems without a desktop environment, the keyring may not be available. In this case, tokens are stored in `~/.config/guts/credentials.toml`.

### Debug Mode

Set the `GIT_TRACE` environment variable to see credential helper interactions:

```bash
GIT_TRACE=1 git clone https://guts.network/owner/repo.git
```

## License

MIT OR Apache-2.0
