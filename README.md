# Stripe CSV

A command-line utility to analyze and process Stripe CSV exports.

## Overview

This tool processes CSV files exported from Stripe and generates summarized analytics.
It's designed to help you quickly analyze transaction data
and generate reports without manual spreadsheet work.

## Features

### Fees Analysis

Analyzes application fees CSV exports from Stripe
and generates a summary report with:

- Total transaction count per account
- Total fees collected per account (in EUR)
- Email addresses associated with each account

The tool automatically groups transactions by user account
and calculates aggregated statistics.

## Usage

### Basic Command

```bash
stripe-csv <FILE> <CSV_TYPE> [OPTIONS]
```

### Analyzing Application Fees

```bash
stripe-csv path/to/fees.csv fees
```

This will create an output file named `fees_out.csv` in the same directory as the input file.

### Custom Output Location

```bash
stripe-csv path/to/fees.csv fees -o path/to/output.csv
```

### Options

- `<FILE>` - Path to the CSV file to analyze (required)
- `<CSV_TYPE>` - Type of analysis to perform (currently supports: `fees`)
- `-o, --output-file <PATH>` - Optional output file path (defaults to `<input>_out.csv`)

## Input Format

### Application Fees CSV

Expected columns in your Stripe export:

```csv
id,Created (UTC),Amount,Amount Refunded,Currency,User ID,User Email,Application ID,Transaction ID
```

Example:

```csv
id,Created (UTC),Amount,Amount Refunded,Currency,User ID,User Email,Application ID,Transaction ID
fee_1ABC123XYZ456789DEF,2025-12-31 14:30,"0,25","0,00",eur,acct_1TEST001ABC123XYZ,user1@example.com,ca_ABC123XYZ456789DEF,ch_3ABC123XYZ456789DEF
fee_1GHI456UVW789012JKL,2025-12-31 10:15,"0,50","0,00",eur,acct_1TEST002GHI456UVW,user2@example.com,ca_ABC123XYZ456789DEF,ch_3GHI456UVW789012JKL
```

## Output Format

### Application Fees Summary

The output CSV contains:

```csv
account_id,email,transaction_count,total_fees_eur
```

Example:

```csv
account_id,email,transaction_count,total_fees_eur
acct_1TEST001ABC123XYZ,user1@example.com,3,1.20
acct_1TEST002GHI456UVW,user2@example.com,2,0.65
```
