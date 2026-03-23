# solana-rbac-onchain

> Superteam Poland Bounty Submission — *Rebuild production backend systems as on-chain Rust programs*
>
> An on-chain **Role-Based Access Control (RBAC)** system built with **Rust + Anchor** on Solana, demonstrating how a fundamental Web2 backend pattern is redesigned using Solana's account model as a distributed state machine.
>
> ## Table of Contents
>
> - [Overview](#overview)
> - - [How This Works in Web2](#how-this-works-in-web2)
>   - - [How This Works on Solana](#how-this-works-on-solana)
>     - - [Architecture and Account Model](#architecture-and-account-model)
>       - - [Program Instructions](#program-instructions)
>         - - [TypeScript Client / CLI](#typescript-client--cli)
>           - - [Tests](#tests)
>             - - [Devnet Deployment](#devnet-deployment)
>               - - [Tradeoffs and Constraints](#tradeoffs-and-constraints)
>                 - - [Running Locally](#running-locally)
>                  
>                   - ## Overview
>                  
>                   - RBAC (Role-Based Access Control) is one of the most fundamental backend patterns in Web2: it controls who can do what on a system. Every enterprise app from AWS IAM to GitHub Teams relies on some variant of RBAC.
>                  
>                   - This project ports the full RBAC pattern to Solana, treating the blockchain as a trustless, censorship-resistant permissioning backend.
>
> Key features:
>
> - Create named resources with an admin authority
> - - Define roles scoped to a resource (e.g., `admin`, `editor`, `viewer`)
>   - - Grant and revoke roles to wallet addresses
>     - - Check permissions on-chain before executing privileged operations
>       - - Fully composable via CPI: any Solana program can call `check_permission`
>         - - Event emission for off-chain indexing and audit trail
>           - - TypeScript SDK plus CLI for operator usage
>            
>             - ## How This Works in Web2
>            
>             - In a typical Web2 backend (Node.js + PostgreSQL):
>            
>             - ```sql
> CREATE TABLE resources (id UUID PRIMARY KEY, name TEXT, admin_user_id UUID);
> CREATE TABLE roles (id UUID PRIMARY KEY, resource_id UUID, name TEXT);
> CREATE TABLE role_assignments (user_id UUID, role_id UUID, PRIMARY KEY(user_id, role_id));
>
> SELECT ra.user_id
> FROM role_assignments ra
> JOIN roles r ON r.id = ra.role_id
> WHERE r.resource_id = $1 AND r.name = $2 AND ra.user_id = $3;
> ```
>
> State lives in a centralized database. An API server mediates all access. Permissions are enforced off-chain. A compromised server equals full permission bypass. Audit logs are optional and mutable.
>
> ## How This Works on Solana
>
> On Solana, state lives in accounts and programs enforce rules. There is no trusted intermediary. Every instruction is verified by the Solana runtime against on-chain state.
>
> | Concept | Web2 | Solana |
> |---|---|---|
> | State storage | PostgreSQL tables | PDA-based accounts |
> | Identity | User ID (UUID) | Wallet public key |
> | Auth enforcement | API middleware | Anchor constraints |
> | Audit log | Optional DB table | On-chain events (immutable) |
> | Composability | REST API call | CPI (Cross-Program Invocation) |
>
> ## Architecture and Account Model
>
> ```
> ResourceAccount  (PDA: ["resource", resource_name, admin_pubkey])
>     admin:       Pubkey
>     name:        String
>     resource_id: [u8; 16]
>     bump:        u8
>
> RoleAccount  (PDA: ["role", resource_key, role_name])
>     resource: Pubkey
>     name:     String
>     bump:     u8
>
> AssignmentAccount  (PDA: ["assignment", role_key, user_pubkey])
>     role:       Pubkey
>     user:       Pubkey
>     granted_at: i64
>     bump:       u8
> ```
>
> **Why PDAs?**
>
> - Deterministic addresses: anyone can derive the address of a permission from public inputs
> - No private key: programs own accounts; no external party can sign for them
> - O(1) lookup: a single `find_program_address` call verifies any permission
>
> ## Program Instructions
>
> | Instruction | Description | Authority |
> |---|---|---|
> | `initialize_resource` | Creates a new resource and sets admin | Any wallet (becomes admin) |
> | `create_role` | Defines a named role on a resource | Resource admin |
> | `grant_role` | Assigns a role to a user | Resource admin |
> | `revoke_role` | Removes a role from a user | Resource admin |
> | `transfer_admin` | Changes the admin of a resource | Current admin |
> | `check_permission` | On-chain permission gate (CPI-friendly) | Any caller |
>
> ## TypeScript Client / CLI
>
> ```bash
> npm install
> ```
>
> ```bash
> # Initialize a resource
> npx ts-node client/cli.ts init-resource --name "my-app" --keypair ~/.config/solana/id.json
>
> # Create a role
> npx ts-node client/cli.ts create-role --resource <RESOURCE_PUBKEY> --role "editor" --keypair ~/.config/solana/id.json
>
> # Grant a role to a user
> npx ts-node client/cli.ts grant-role --resource <RESOURCE_PUBKEY> --role "editor" --user <USER_PUBKEY> --keypair ~/.config/solana/id.json
>
> # Check permission (exit 0 = granted, 1 = denied)
> npx ts-node client/cli.ts check --resource <RESOURCE_PUBKEY> --role "editor" --user <USER_PUBKEY>
>
> # Revoke a role
> npx ts-node client/cli.ts revoke-role --resource <RESOURCE_PUBKEY> --role "editor" --user <USER_PUBKEY> --keypair ~/.config/solana/id.json
> ```
>
> ## Tests
>
> Run with: `anchor test`
>
> Tests cover: `initialize_resource`, `create_role`, `grant_role`, `check_permission` (pass and fail cases), `revoke_role`, `transfer_admin`, and unauthorized grant rejection.
>
> ## Devnet Deployment
>
> Program ID: `RBACxqmGJv8sF3FjQLZ7neQhQdaVHBFfFkCLmqNBjkG`
>
> Verify:
>
> ```bash
> solana program show RBACxqmGJv8sF3FjQLZ7neQhQdaVHBFfFkCLmqNBjkG --url devnet
> ```
>
> ## Tradeoffs and Constraints
>
> | Dimension | Web2 (PostgreSQL) | Solana (On-chain) |
> |---|---|---|
> | State mutability | Full CRUD any time | Accounts closed, not arbitrarily mutated |
> | Identity | Email / UUID | Wallet pubkey (cryptographic, self-sovereign) |
> | Enumeration | SELECT all rows | Requires `getProgramAccounts` indexing |
> | Cost | Server + DB compute | Rent per account (~0.002 SOL each) |
> | Auditability | Optional mutable logs | Every state change is immutable on-chain event |
> | Censorship resistance | None | Only admin key can modify roles |
> | Composability | REST API | CPI: any on-chain program calls `check_permission` |
>
> Key design decisions:
>
> - PDA seeds use resource name + admin pubkey so two admins cannot overwrite each other
> - - `AssignmentAccount` is closed (not soft-deleted) on revoke, returning rent to admin
>   - - `check_permission` works as CPI verifier: success means user has the role, no return value needed
>     - - Role names stored on-chain (not just IDs) for better debuggability and composability
>      
>       - ## Running Locally
>      
>       - Prerequisites: Rust, Solana CLI >= 1.18, Anchor CLI >= 0.30, Node.js >= 18
>      
>       - ```bash
> git clone https://github.com/NandaCardinaly/solana-rbac-onchain
> cd solana-rbac-onchain
> npm install
> anchor build
> anchor test
> solana config set --url devnet
> anchor deploy --provider.cluster devnet
> ```
>
> ## Project Structure
>
> ```
> solana-rbac-onchain/
> ├── programs/rbac/
> │   ├── Cargo.toml
> │   └── src/
> │       ├── lib.rs
> │       ├── state.rs
> │       ├── errors.rs
> │       ├── events.rs
> │       └── instructions/
> │           ├── mod.rs
> │           ├── initialize_resource.rs
> │           ├── create_role.rs
> │           ├── grant_role.rs
> │           ├── revoke_role.rs
> │           ├── transfer_admin.rs
> │           └── check_permission.rs
> ├── client/
> │   ├── cli.ts
> │   └── rbac-client.ts
> ├── tests/
> │   └── rbac.ts
> ├── Anchor.toml
> ├── Cargo.toml
> ├── package.json
> └── tsconfig.json
> ```
>
> ---
>
> Fernanda Cardinaly — [@NandaCardinaly](https://github.com/NandaCardinaly)
