import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider } from "@coral-xyz/anchor";
import { Connection, PublicKey, Keypair, clusterApiUrl } from "@solana/web3.js";
import { Rbac } from "../target/types/rbac";

export const PROGRAM_ID = new PublicKey("RBACxqmGJv8sF3FjQLZ7neQhQdaVHBFfFkCLmqNBjkG");

export class RbacClient {
      private program: Program<Rbac>;
      private provider: AnchorProvider;

    constructor(provider: AnchorProvider) {
              this.provider = provider;
              this.program = new anchor.Program(
                            require("../target/idl/rbac.json"),
                            provider
                        ) as Program<Rbac>;
    }

    static async fromKeypair(
              keypair: Keypair,
              cluster: "devnet" | "mainnet-beta" | "localnet" = "devnet"
          ): Promise<RbacClient> {
              const url =
                            cluster === "localnet" ? "http://localhost:8899" : clusterApiUrl(cluster);
              const connection = new Connection(url, "confirmed");
              const wallet = new anchor.Wallet(keypair);
              const provider = new AnchorProvider(connection, wallet, { commitment: "confirmed" });
              return new RbacClient(provider);
    }

    resourcePDA(name: string, adminKey: PublicKey): [PublicKey, number] {
              return PublicKey.findProgramAddressSync(
                            [Buffer.from("resource"), Buffer.from(name), adminKey.toBuffer()],
                            PROGRAM_ID
                        );
    }

    rolePDA(resourceKey: PublicKey, roleName: string): [PublicKey, number] {
              return PublicKey.findProgramAddressSync(
                            [Buffer.from("role"), resourceKey.toBuffer(), Buffer.from(roleName)],
                            PROGRAM_ID
                        );
    }

    assignmentPDA(roleKey: PublicKey, userKey: PublicKey): [PublicKey, number] {
              return PublicKey.findProgramAddressSync(
                            [Buffer.from("assignment"), roleKey.toBuffer(), userKey.toBuffer()],
                            PROGRAM_ID
                        );
    }

    async initializeResource(name: string, resourceId: number[] = []): Promise<string> {
              if (resourceId.length === 0) {
                            resourceId = Array.from(crypto.getRandomValues(new Uint8Array(16)));
              }
              const admin = this.provider.wallet.publicKey;
              const [resource] = this.resourcePDA(name, admin);
              return this.program.methods
                  .initializeResource(name, resourceId)
                  .accounts({ resource, admin, systemProgram: anchor.web3.SystemProgram.programId })
                  .rpc();
    }

    async createRole(resourceKey: PublicKey, roleName: string): Promise<string> {
              const admin = this.provider.wallet.publicKey;
              const [role] = this.rolePDA(resourceKey, roleName);
              return this.program.methods
                  .createRole(roleName)
                  .accounts({
                                    resource: resourceKey,
                                    role,
                                    admin,
                                    systemProgram: anchor.web3.SystemProgram.programId,
                  })
                  .rpc();
    }

    async grantRole(
              resourceKey: PublicKey,
              roleKey: PublicKey,
              userKey: PublicKey
          ): Promise<string> {
              const admin = this.provider.wallet.publicKey;
              const [assignment] = this.assignmentPDA(roleKey, userKey);
              return this.program.methods
                  .grantRole()
                  .accounts({
                                    resource: resourceKey,
                                    role: roleKey,
                                    assignment,
                                    user: userKey,
                                    admin,
                                    systemProgram: anchor.web3.SystemProgram.programId,
                  })
                  .rpc();
    }

    async revokeRole(
              resourceKey: PublicKey,
              roleKey: PublicKey,
              userKey: PublicKey
          ): Promise<string> {
              const admin = this.provider.wallet.publicKey;
              const [assignment] = this.assignmentPDA(roleKey, userKey);
              return this.program.methods
                  .revokeRole()
                  .accounts({ resource: resourceKey, role: roleKey, assignment, user: userKey, admin })
                  .rpc();
    }

    async checkPermission(roleKey: PublicKey, userKey: PublicKey): Promise<boolean> {
              const [assignment] = this.assignmentPDA(roleKey, userKey);
              try {
                            await this.program.methods
                                .checkPermission()
                                .accounts({ role: roleKey, assignment, user: userKey })
                                .rpc();
                            return true;
              } catch {
                            return false;
              }
    }

    async hasRole(roleKey: PublicKey, userKey: PublicKey): Promise<boolean> {
              const [assignment] = this.assignmentPDA(roleKey, userKey);
              const acct = await this.provider.connection.getAccountInfo(assignment);
              return acct !== null;
    }

    async getResource(resourceKey: PublicKey) {
              return this.program.account.resourceAccount.fetch(resourceKey);
    }

    async getRole(roleKey: PublicKey) {
              return this.program.account.roleAccount.fetch(roleKey);
    }

    async transferAdmin(resourceKey: PublicKey, newAdmin: PublicKey): Promise<string> {
              const admin = this.provider.wallet.publicKey;
              return this.program.methods
                  .transferAdmin(newAdmin)
                  .accounts({ resource: resourceKey, admin })
                  .rpc();
    }
}
