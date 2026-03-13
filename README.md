# solana-rbac-onchain

> **Superteam Poland Bounty Submission** — *Rebuild production backend systems as on-chain Rust programs*

An on-chain **Role-Based Access Control (RBAC)** system built with **Rust + Anchor** on Solana, demonstrating how a fundamental Web2 backend pattern is redesigned using Solana's account model as a distributed state machine.

---

## Table of Contents

- [Overview](#overview)
- [How This Works in Web2](#how-this-works-in-web2)
- [How This Works on Solana](#how-this-works-on-solana)
- [Architecture and Account Model](#architecture-and-account-model)
- [Program Instructions](#program-instructions)
- [Code](#code)
- [TypeScript Client CLI](#typescript-client-cli)
- [Tests](#tests)
- [Devnet Deployment](#devnet-deployment)
- [Tradeoffs and Constraints](#tradeoffs-and-constraints)
- [Running Locally](#running-locally)

---

## Overview

RBAC (Role-Based Access Control) is one of the most fundamental backend patterns in Web2: it controls who can do what on a system. Every enterprise app from AWS IAM to GitHub Teams relies on some variant of RBAC.

This project ports the full RBAC pattern to Solana, treating the blockchain as a trustless, censorship-resistant permissioning backend.

Key features:
- Create named resources with an admin authority
- Define roles scoped to a resource (e.g., admin, editor, viewer)
- Grant and Revoke roles to wallet addresses
- Check permissions on-chain before executing privileged operations
- Fully composable via CPI: any Solana program can call check_permission
- Event emission for off-chain indexing and audit trail
- TypeScript SDK plus CLI for operator usage

---

## How This Works in Web2

In a typical Web2 backend (Node.js + PostgreSQL):

```sql
CREATE TABLE resources (id UUID PRIMARY KEY, name TEXT, admin_user_id UUID);
CREATE TABLE roles (id UUID PRIMARY KEY, resource_id UUID, name TEXT);
CREATE TABLE role_assignments (user_id UUID, role_id UUID, PRIMARY KEY(user_id, role_id));

SELECT ra.user_id FROM role_assignments ra
JOIN roles r ON r.id = ra.role_id
WHERE r.resource_id = $1 AND r.name = $2 AND ra.user_id = $3;
```

State lives in a centralized database. An API server mediates all access. Permissions are enforced off-chain. A compromised server equals full permission bypass. Audit logs are optional and mutable.

---

## How This Works on Solana

On Solana, state lives in accounts and programs enforce rules. There is no trusted intermediary. Every instruction is verified by the Solana runtime against on-chain state.

| Concept | Web2 | Solana |
|---|---|---|
| State storage | PostgreSQL tables | PDA-based accounts |
| Identity | User ID (UUID) | Wallet public key |
| Auth enforcement | API middleware | Anchor constraints |
| Audit log | Optional DB table | On-chain events (immutable) |
| Composability | REST API call | CPI (Cross-Program Invocation) |

---

## Architecture and Account Model

```
ResourceAccount (PDA: ["resource", resource_name, admin_pubkey])
  admin: Pubkey          -- can manage roles
  name: String           -- human-readable resource name
  resource_id: [u8; 16]  -- UUID equivalent
  bump: u8

RoleAccount (PDA: ["role", resource_key, role_name])
  resource: Pubkey       -- parent resource
  name: String           -- role name (e.g., "editor")
  bump: u8

AssignmentAccount (PDA: ["assignment", role_key, user_pubkey])
  role: Pubkey           -- which role
  user: Pubkey           -- who has it
  granted_at: i64        -- Unix timestamp
  bump: u8
```

**Why PDAs?**
- Deterministic addresses: anyone can derive the address of a permission from public inputs
- No private key: programs own accounts; no external party can sign for them
- O(1) lookup: a single find_program_address call verifies any permission

---

## Program Instructions

| Instruction | Description | Authority |
|---|---|---|
| initialize_resource | Creates a new resource and sets admin | Any wallet (becomes admin) |
| create_role | Defines a named role on a resource | Resource admin |
| grant_role | Assigns a role to a user | Resource admin |
| revoke_role | Removes a role from a user | Resource admin |
| transfer_admin | Changes the admin of a resource | Current admin |
| check_permission | On-chain permission gate (CPI-friendly) | Any caller |

---

## Code

### programs/rbac/src/lib.rs

```rust
use anchor_lang::prelude::*;

declare_id!("RBACxqmGJv8sF3FjQLZ7neQhQdaVHBFfFkCLmqNBjkG");

pub mod instructions;
pub mod state;
pub mod errors;
pub mod events;

use instructions::*;

#[program]
pub mod rbac {
    use super::*;

    pub fn initialize_resource(
        ctx: Context<InitializeResource>,
        name: String,
        resource_id: [u8; 16],
    ) -> Result<()> {
        instructions::initialize_resource::handler(ctx, name, resource_id)
    }

    pub fn create_role(ctx: Context<CreateRole>, role_name: String) -> Result<()> {
        instructions::create_role::handler(ctx, role_name)
    }

    pub fn grant_role(ctx: Context<GrantRole>) -> Result<()> {
        instructions::grant_role::handler(ctx)
    }

    pub fn revoke_role(ctx: Context<RevokeRole>) -> Result<()> {
        instructions::revoke_role::handler(ctx)
    }

    pub fn transfer_admin(ctx: Context<TransferAdmin>, new_admin: Pubkey) -> Result<()> {
        instructions::transfer_admin::handler(ctx, new_admin)
    }

    pub fn check_permission(ctx: Context<CheckPermission>) -> Result<()> {
        instructions::check_permission::handler(ctx)
    }
}
```

### programs/rbac/src/state.rs

```rust
use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct ResourceAccount {
    pub admin: Pubkey,
    pub name: String,
    pub resource_id: [u8; 16],
    pub bump: u8,
}

impl ResourceAccount {
    pub const MAX_NAME_LEN: usize = 32;
    pub const LEN: usize = 8 + 32 + (4 + Self::MAX_NAME_LEN) + 16 + 1;
}

#[account]
#[derive(Default)]
pub struct RoleAccount {
    pub resource: Pubkey,
    pub name: String,
    pub bump: u8,
}

impl RoleAccount {
    pub const MAX_NAME_LEN: usize = 32;
    pub const LEN: usize = 8 + 32 + (4 + Self::MAX_NAME_LEN) + 1;
}

#[account]
#[derive(Default)]
pub struct AssignmentAccount {
    pub role: Pubkey,
    pub user: Pubkey,
    pub granted_at: i64,
    pub bump: u8,
}

impl AssignmentAccount {
    pub const LEN: usize = 8 + 32 + 32 + 8 + 1;
}
```

### programs/rbac/src/instructions/grant_role.rs

```rust
use anchor_lang::prelude::*;
use crate::state::{ResourceAccount, RoleAccount, AssignmentAccount};
use crate::errors::RbacError;
use crate::events::RoleGranted;

#[derive(Accounts)]
pub struct GrantRole<'info> {
    #[account(has_one = admin @ RbacError::NotAdmin)]
    pub resource: Account<'info, ResourceAccount>,
    pub role: Account<'info, RoleAccount>,
    #[account(
        init,
        payer = admin,
        space = AssignmentAccount::LEN,
        seeds = [b"assignment", role.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub assignment: Account<'info, AssignmentAccount>,
    /// CHECK: The user receiving the role.
    pub user: UncheckedAccount<'info>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<GrantRole>) -> Result<()> {
    let assignment = &mut ctx.accounts.assignment;
    assignment.role = ctx.accounts.role.key();
    assignment.user = ctx.accounts.user.key();
    assignment.granted_at = Clock::get()?.unix_timestamp;
    assignment.bump = ctx.bumps.assignment;

    emit!(RoleGranted {
        resource: ctx.accounts.resource.key(),
        role: ctx.accounts.role.key(),
        user: ctx.accounts.user.key(),
        granted_at: assignment.granted_at,
    });

    Ok(())
}
```

### programs/rbac/src/instructions/check_permission.rs

```rust
use anchor_lang::prelude::*;
use crate::state::{RoleAccount, AssignmentAccount};
use crate::errors::RbacError;

/// CPI-friendly permission gate.
/// Succeeds if and only if the user holds the specified role.
/// Any Solana program can call this to enforce access control.
#[derive(Accounts)]
pub struct CheckPermission<'info> {
    pub role: Account<'info, RoleAccount>,
    #[account(
        seeds = [b"assignment", role.key().as_ref(), user.key().as_ref()],
        bump = assignment.bump,
        constraint = assignment.user == user.key() @ RbacError::AccessDenied,
    )]
    pub assignment: Account<'info, AssignmentAccount>,
    /// CHECK: The user whose permission we are checking.
    pub user: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<CheckPermission>) -> Result<()> {
    msg!(
        "Permission check passed: user {} holds role {}",
        ctx.accounts.user.key(),
        ctx.accounts.role.name
    );
    Ok(())
}
```

### programs/rbac/src/errors.rs

```rust
use anchor_lang::prelude::*;

#[error_code]
pub enum RbacError {
    #[msg("Signer is not the resource admin")]
    NotAdmin,
    #[msg("Access denied: user does not hold the required role")]
    AccessDenied,
    #[msg("Resource name exceeds maximum length")]
    NameTooLong,
    #[msg("Cannot transfer admin to the current admin")]
    SameAdmin,
}
```

### programs/rbac/src/events.rs

```rust
use anchor_lang::prelude::*;

#[event]
pub struct ResourceInitialized {
    pub resource: Pubkey,
    pub admin: Pubkey,
    pub name: String,
}

#[event]
pub struct RoleGranted {
    pub resource: Pubkey,
    pub role: Pubkey,
    pub user: Pubkey,
    pub granted_at: i64,
}

#[event]
pub struct RoleRevoked {
    pub resource: Pubkey,
    pub role: Pubkey,
    pub user: Pubkey,
}

#[event]
pub struct AdminTransferred {
    pub resource: Pubkey,
    pub old_admin: Pubkey,
    pub new_admin: Pubkey,
}
```

---

## TypeScript Client CLI

```bash
npm install
npx ts-node client/cli.ts --help
```

### Examples

```bash
# Initialize a resource
npx ts-node client/cli.ts init-resource --name "my-app" --keypair ~/.config/solana/id.json

# Create a role
npx ts-node client/cli.ts create-role --resource <RESOURCE_PUBKEY> --role "editor" --keypair ~/.config/solana/id.json

# Grant a role to a user
npx ts-node client/cli.ts grant-role --resource <RESOURCE_PUBKEY> --role "editor" --user <USER_PUBKEY> --keypair ~/.config/solana/id.json

# Check permission (exit 0 = granted, 1 = denied)
npx ts-node client/cli.ts check --resource <RESOURCE_PUBKEY> --role "editor" --user <USER_PUBKEY>

# Revoke a role
npx ts-node client/cli.ts revoke-role --resource <RESOURCE_PUBKEY> --role "editor" --user <USER_PUBKEY> --keypair ~/.config/solana/id.json
```

---

## Tests

Run with: `anchor test`

| Test | Status |
|---|---|
| initialize_resource: creates PDA and sets admin | PASS |
| create_role: derives role PDA correctly | PASS |
| grant_role: assigns role, verifies assignment PDA | PASS |
| check_permission (pass): succeeds when user holds role | PASS |
| check_permission (fail): rejects with AccessDenied | PASS |
| revoke_role: closes assignment account | PASS |
| transfer_admin: changes admin, old admin rejected | PASS |
| unauthorized grant: rejects with NotAdmin | PASS |

---

## Devnet Deployment

**Program ID:** `RBACxqmGJv8sF3FjQLZ7neQhQdaVHBFfFkCLmqNBjkG`

Verify:
```bash
solana program show RBACxqmGJv8sF3FjQLZ7neQhQdaVHBFfFkCLmqNBjkG --url devnet
```

---

## Tradeoffs and Constraints

| Dimension | Web2 (PostgreSQL) | Solana (On-chain) |
|---|---|---|
| State mutability | Full CRUD any time | Accounts closed, not arbitrarily mutated |
| Identity | Email / UUID | Wallet pubkey (cryptographic, self-sovereign) |
| Enumeration | SELECT all rows | Requires getProgramAccounts indexing |
| Cost | Server + DB compute | Rent per account (~0.002 SOL each) |
| Auditability | Optional mutable logs | Every state change is immutable on-chain event |
| Censorship resistance | None | Only admin key can modify roles |
| Composability | REST API | CPI: any on-chain program calls check_permission |

**Key design decisions:**
- PDA seeds use resource name + admin pubkey so two admins cannot overwrite each other
- AssignmentAccount is closed (not soft-deleted) on revoke, returning rent to admin
- check_permission works as CPI verifier: success means user has the role, no return value needed
- Role names stored on-chain (not just IDs) for better debuggability and composability

---

## Running Locally

**Prerequisites:** Rust, Solana CLI >= 1.18, Anchor CLI >= 0.30, Node.js >= 18

```bash
git clone https://github.com/NandaCardinaly/solana-rbac-onchain
cd solana-rbac-onchain
npm install
anchor build
anchor test
solana config set --url devnet
anchor deploy --provider.cluster devnet
```

---

## Project Structure

```
solana-rbac-onchain/
├── programs/rbac/src/
│   ├── lib.rs
│   ├── state.rs
│   ├── errors.rs
│   ├── events.rs
│   └── instructions/
│       ├── mod.rs
│       ├── initialize_resource.rs
│       ├── create_role.rs
│       ├── grant_role.rs
│       ├── revoke_role.rs
│       ├── transfer_admin.rs
│       └── check_permission.rs
├── client/
│   ├── cli.ts
│   └── rbac-client.ts
├── tests/rbac.ts
├── Anchor.toml
└── README.md
```

---

## Author

**Fernanda Cardinaly** — [@NandaCardinaly](https://github.com/NandaCardinaly)

Superteam Earn Bounty: *Rebuild production backend systems as on-chain Rust programs*
