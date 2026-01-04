# Sparebank1 to YNAB

A Rust-based tool to automatically sync transactions from SpareBank 1 (Norwegian bank) to You Need A Budget (YNAB).

## Overview

This project provides two binaries:
- **sparebank1-to-ynab-setup**: Interactive setup wizard to configure the integration
- **sparebank1-to-ynab-sync**: Synchronization tool to fetch transactions and import them to YNAB

The sync tool is designed to run periodically (e.g., via cron or as a scheduled container) to keep your YNAB budget up-to-date with your SpareBank 1 transactions.

## Features

- 🔐 OAuth authentication with SpareBank 1 API
- 🔄 Automatic token refresh handling
- 💰 Syncs transactions from multiple SpareBank 1 accounts
- 🎯 Maps SpareBank 1 accounts to YNAB accounts
- 🔍 Duplicate detection to prevent re-importing transactions
- 🐳 Docker support for easy deployment
- 📦 GitHub Container Registry releases

## Prerequisites

### SpareBank 1 API Access
You need to register an application with SpareBank 1 to get:
- Client ID
- Client Secret
- Financial Institution ID (`finInst`)

Contact SpareBank 1 or visit their developer portal for API access.

### YNAB Personal Access Token
Generate a personal access token from your YNAB account:
1. Go to [YNAB Developer Settings](https://app.youneedabudget.com/settings/developer)
2. Create a new personal access token
3. Save the token securely

## Installation

### Building from Source

```bash
cargo build --release
```

This will create two binaries in `target/release/`:
- `sparebank1-to-ynab-setup`
- `sparebank1-to-ynab-sync`

### Using Docker

Pull the pre-built image from GitHub Container Registry:

```bash
docker pull ghcr.io/bjorngi/sparebank1-to-ynab/sparebank1-to-ynab-sync:latest
```

## Setup

### Step 1: Run the Setup Wizard

The setup wizard will:
1. Open your browser for SpareBank 1 OAuth authentication
2. Fetch your SpareBank 1 accounts
3. Let you select a YNAB budget
4. Map each SpareBank 1 account to a YNAB account
5. Generate configuration files

Run the setup:

```bash
./sparebank1-to-ynab-setup <CLIENT_ID> <CLIENT_SECRET> <FIN_INST> <YNAB_ACCESS_TOKEN>
```

**Arguments:**
- `CLIENT_ID`: Your SpareBank 1 API client ID
- `CLIENT_SECRET`: Your SpareBank 1 API client secret
- `FIN_INST`: Your SpareBank 1 financial institution ID
- `YNAB_ACCESS_TOKEN`: Your YNAB personal access token

### Step 2: Configuration Files

The setup wizard creates two files:

**`budget.env`** - Environment configuration:
```env
SPAREBANK1_CLIENT_ID=your_client_id
SPAREBANK1_CLIENT_SECRET=your_client_secret
SPAREBANK1_FIN_INST=your_fin_inst
YNAB_BUDGET_ID=your_budget_id
YNAB_ACCESS_TOKEN=your_ynab_token
INITIAL_REFRESH_TOKEN=your_refresh_token
ACCOUNT_CONFIG_PATH=/path/to/accounts.json
REFRESH_TOKEN_FILE_PATH=refresh_token.txt
```

**`accounts.json`** - Account mapping:
```json
{
  "sparebank1_account_key_1": "ynab_account_id_1",
  "sparebank1_account_key_2": "ynab_account_id_2"
}
```

**`refresh_token.txt`** - OAuth refresh token (auto-updated)

## Usage

### Manual Sync

Run the sync tool manually:

```bash
./sparebank1-to-ynab-sync
```

The tool will:
1. Load configuration from `budget.env`
2. Refresh the SpareBank 1 access token if needed
3. Fetch recent transactions from configured accounts
4. Import transactions to YNAB with duplicate detection
5. Display a summary of imported and duplicate transactions

### Docker

Run sync with Docker:

```bash
docker run --rm \
  -v $(pwd)/budget.env:/app/.env \
  -v $(pwd)/accounts.json:/app/accounts.json \
  -v $(pwd)/refresh_token.txt:/app/refresh_token.txt \
  ghcr.io/bjorngi/sparebank1-to-ynab/sparebank1-to-ynab-sync:latest
```

### Automated Sync (Cron)

Add to your crontab to run every hour:

```cron
0 * * * * cd /path/to/sparebank1-to-ynab && ./sparebank1-to-ynab-sync >> sync.log 2>&1
```

## How It Works

### Transaction Import

1. **Fetch**: The sync tool fetches transactions from SpareBank 1 API for all configured accounts
2. **Transform**: Transactions are converted to YNAB format with:
   - Date conversion (timestamp to YYYY-MM-DD in Oslo timezone)
   - Amount conversion (float to milliunits: NOK × 1000)
   - Import ID generation for duplicate detection: `SB1:{amount}:{date}:{occurrence}`
3. **Import**: Transactions are sent to YNAB's bulk import API
4. **Deduplicate**: YNAB automatically skips transactions with duplicate import IDs

### Token Management

- Initial OAuth flow in setup generates access and refresh tokens
- Access tokens expire after a period
- The sync tool automatically refreshes tokens using the refresh token
- New refresh tokens are saved to `refresh_token.txt` after each refresh


### Logging

The application uses structured logging with configurable log levels. By default, logs at `info` level and above are displayed.

**Control log level with the `RUST_LOG` environment variable:**

```bash
# Show all logs (including debug messages)
RUST_LOG=debug ./sparebank1-to-ynab-sync

# Show only warnings and errors
RUST_LOG=warn ./sparebank1-to-ynab-sync

# Show specific module logs
RUST_LOG=sparebank1_to_ynab::ynab=debug ./sparebank1-to-ynab-sync

# Default (info level)
./sparebank1-to-ynab-sync
```

**Log levels:**
- `error` - Critical errors only
- `warn` - Warnings and errors
- `info` - General information, warnings, and errors (default)
- `debug` - Detailed debugging information
- `trace` - Very verbose trace-level logging

**Example output:**
```
2024-01-15T10:30:00.123Z  INFO sparebank1_to_ynab::sync: Starting SpareBank1 to YNAB sync
2024-01-15T10:30:00.456Z  INFO sparebank1_to_ynab::config: Configuration loaded successfully
2024-01-15T10:30:00.789Z  INFO sparebank1_to_ynab::auth_data: Successfully refreshed access token
2024-01-15T10:30:01.234Z  INFO sparebank1_to_ynab::sparebanken1: Successfully fetched 25 transactions
2024-01-15T10:30:02.567Z  INFO sparebank1_to_ynab::ynab: Successfully added 20 transactions to YNAB
2024-01-15T10:30:02.568Z  INFO sparebank1_to_ynab::sync: Added 20 new transactions
2024-01-15T10:30:02.568Z  INFO sparebank1_to_ynab::sync: Skipped 5 duplicate transactions
```

**With Docker:**
```bash
docker run --rm \
  -e RUST_LOG=debug \
  -v $(pwd)/budget.env:/app/.env \
  -v $(pwd)/accounts.json:/app/accounts.json \
  -v $(pwd)/refresh_token.txt:/app/refresh_token.txt \
  ghcr.io/bjorngi/sparebank1-to-ynab/sparebank1-to-ynab-sync:latest
```

## Development

### Build

```bash
cargo build
```

### Run Tests

```bash
cargo test
```

### Run Locally

Set up `.env` file (or use `budget.env`):

```bash
cp budget.env .env
cargo run --bin sparebank1-to-ynab-sync
```

## Project Structure

```
src/
├── bin/
│   ├── setup.rs           # Interactive setup wizard
│   └── sync.rs            # Transaction sync tool
├── account_config.rs      # Account mapping configuration
├── auth_data.rs           # OAuth token management
├── config.rs              # Application configuration
├── sparebanken1.rs        # SpareBank 1 API client
├── ynab.rs                # YNAB API client
└── lib.rs                 # Library exports
```

## Troubleshooting

### "Failed to get access_token"
- Check that your `refresh_token.txt` exists and is valid
- Try running the setup again to get a fresh refresh token

### "Request error" when syncing
- Verify your SpareBank 1 credentials are correct
- Check that your access token hasn't been revoked
- Ensure your financial institution ID is correct

### Transactions not appearing in YNAB
- Verify the account mapping in `accounts.json` is correct
- Check that the YNAB account IDs match your actual accounts
- Review the sync output for duplicate transaction messages

### Docker: "No such file or directory"
- Ensure volume mounts point to existing files
- Check that paths are absolute or relative to the correct directory

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

- [SpareBank 1 API](https://api.sparebank1.no)
- [YNAB API](https://api.youneedabudget.com)
