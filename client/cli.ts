import * as fs from "fs";
import * as path from "path";
import { PublicKey, Keypair } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import { AnchorProvider } from "@coral-xyz/anchor";
import { Connection, clusterApiUrl } from "@solana/web3.js";
import { RbacClient } from "./rbac-client";

const args = process.argv.slice(2);

function usage() {
  console.log(`
              Usage: npx ts-node client/cli.ts <command> [options]

Commands:
  init-resource --name <NAME> --keypair <PATH>
  create-role   --resource <PUBKEY> --role <NAME> --keypair <PATH>
  grant-role    --resource <PUBKEY> --role <NAME> --user <PUBKEY> --keypair <PATH>
  revoke-role   --resource <PUBKEY> --role <NAME> --user <PUBKEY> --keypair <PATH>
  check         --resource <PUBKEY> --role <NAME> --user <PUBKEY>
`);
  process.exit(1);
}

function getArg(name: string): string | undefined {
  const idx = args.indexOf(`--${name}`);
  return idx !== -1 ? args[idx + 1] : undefined;
}

function requireArg(name: string): string {
  const val = getArg(name);
  if (!val) {
    console.error(`Missing required argument: --${name}`);
    usage();
  }
  return val!;
}

function loadKeypair(kpPath: string): Keypair {
  const raw = fs.readFileSync(path.resolve(kpPath), "utf8");
  return Keypair.fromSecretKey(Buffer.from(JSON.parse(raw)));
}

async function main() {
  const command = args[0];
  if (!command) usage();

  const cluster = (getArg("cluster") as any) || "devnet";
  const url = cluster === "localnet" ? "http://localhost:8899" : clusterApiUrl(cluster);

  const kpPath = getArg("keypair") || `${process.env.HOME}/.config/solana/id.json`;
  const keypair = loadKeypair(kpPath);
  const connection = new Connection(url, "confirmed");
  const wallet = new anchor.Wallet(keypair);
  const provider = new AnchorProvider(connection, wallet, { commitment: "confirmed" });
  const client = new RbacClient(provider);

  switch (command) {
    case "init-resource": {
      const name = requireArg("name");
      const tx = await client.initializeResource(name);
      const [pda] = client.resourcePDA(name, keypair.publicKey);
      console.log(`tx: ${tx}`);
      console.log(`resource: ${pda.toBase58()}`);
      break;
    }
    case "create-role": {
      const resource = new PublicKey(requireArg("resource"));
      const role = requireArg("role");
      const tx = await client.createRole(resource, role);
      const [pda] = client.rolePDA(resource, role);
      console.log(`tx: ${tx}`);
      console.log(`role: ${pda.toBase58()}`);
      break;
    }
    case "grant-role": {
      const resource = new PublicKey(requireArg("resource"));
      const roleName = requireArg("role");
      const user = new PublicKey(requireArg("user"));
      const [rolePDA] = client.rolePDA(resource, roleName);
      const tx = await client.grantRole(resource, rolePDA, user);
      console.log(`tx: ${tx}`);
      break;
    }
    case "revoke-role": {
      const resource = new PublicKey(requireArg("resource"));
      const roleName = requireArg("role");
      const user = new PublicKey(requireArg("user"));
      const [rolePDA] = client.rolePDA(resource, roleName);
      const tx = await client.revokeRole(resource, rolePDA, user);
      console.log(`tx: ${tx}`);
      break;
    }
    case "check": {
      const resource = new PublicKey(requireArg("resource"));
      const roleName = requireArg("role");
      const user = new PublicKey(requireArg("user"));
      const [rolePDA] = client.rolePDA(resource, roleName);
      const has = await client.hasRole(rolePDA, user);
      console.log(has ? "granted" : "denied");
      process.exit(has ? 0 : 1);
      break;
    }
    default:
      console.error(`Unknown command: ${command}`);
      usage();
  }
}

main().catch((e) => {
                   console.error(e);
                   process.exit(1);
                 });
