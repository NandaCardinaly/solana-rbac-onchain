import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Rbac } from "../target/types/rbac";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { assert, expect } from "chai";

describe("solana-rbac-onchain", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Rbac as Program<Rbac>;

  const admin = provider.wallet as anchor.Wallet;
  const user = Keypair.generate();
  const stranger = Keypair.generate();

  const resourceName = "test-app";
  const resourceId = Array.from({ length: 16 }, () => Math.floor(Math.random() * 256));
  const roleName = "editor";

  let resourcePDA: PublicKey;
  let rolePDA: PublicKey;

  before(async () => {
    // Airdrop to user for testing
    await provider.connection.requestAirdrop(user.publicKey, 1e9);
    await provider.connection.requestAirdrop(stranger.publicKey, 1e9);
    await new Promise((r) => setTimeout(r, 1000));

    [resourcePDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("resource"), Buffer.from(resourceName), admin.publicKey.toBuffer()],
      program.programId
    );
    [rolePDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("role"), resourcePDA.toBuffer(), Buffer.from(roleName)],
      program.programId
    );
  });

  it("initializes a resource", async () => {
    await program.methods
      .initializeResource(resourceName, resourceId)
      .accounts({ resource: resourcePDA, admin: admin.publicKey, systemProgram: SystemProgram.programId })
      .rpc();

    const resource = await program.account.resourceAccount.fetch(resourcePDA);
    assert.equal(resource.admin.toBase58(), admin.publicKey.toBase58());
    assert.equal(resource.name, resourceName);
  });

  it("creates a role", async () => {
    await program.methods
      .createRole(roleName)
      .accounts({ resource: resourcePDA, role: rolePDA, admin: admin.publicKey, systemProgram: SystemProgram.programId })
      .rpc();

    const role = await program.account.roleAccount.fetch(rolePDA);
    assert.equal(role.resource.toBase58(), resourcePDA.toBase58());
    assert.equal(role.name, roleName);
  });

  it("grants a role to a user", async () => {
    const [assignmentPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("assignment"), rolePDA.toBuffer(), user.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .grantRole()
      .accounts({
        resource: resourcePDA,
        role: rolePDA,
        assignment: assignmentPDA,
        user: user.publicKey,
        admin: admin.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const assignment = await program.account.assignmentAccount.fetch(assignmentPDA);
    assert.equal(assignment.user.toBase58(), user.publicKey.toBase58());
    assert.equal(assignment.role.toBase58(), rolePDA.toBase58());
    assert.isAbove(assignment.grantedAt.toNumber(), 0);
  });

  it("check_permission passes when user holds the role", async () => {
    const [assignmentPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("assignment"), rolePDA.toBuffer(), user.publicKey.toBuffer()],
      program.programId
    );

    // Should not throw
    await program.methods
      .checkPermission()
      .accounts({ role: rolePDA, assignment: assignmentPDA, user: user.publicKey })
      .rpc();
  });

  it("check_permission fails when user does NOT hold the role", async () => {
    const [fakeAssignmentPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("assignment"), rolePDA.toBuffer(), stranger.publicKey.toBuffer()],
      program.programId
    );

    try {
      await program.methods
        .checkPermission()
        .accounts({ role: rolePDA, assignment: fakeAssignmentPDA, user: stranger.publicKey })
        .rpc();
      assert.fail("Expected AccessDenied error");
    } catch (e: any) {
      expect(e.message).to.include("AccessDenied");
    }
  });

  it("revokes a role from a user", async () => {
    const [assignmentPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("assignment"), rolePDA.toBuffer(), user.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .revokeRole()
      .accounts({
        resource: resourcePDA,
        role: rolePDA,
        assignment: assignmentPDA,
        user: user.publicKey,
        admin: admin.publicKey,
      })
      .rpc();

    // Account should be closed
    const acct = await provider.connection.getAccountInfo(assignmentPDA);
    assert.isNull(acct, "Assignment account should be closed after revoke");
  });

  it("rejects grant from a non-admin", async () => {
    const newRole = "viewer";
    const [newRolePDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("role"), resourcePDA.toBuffer(), Buffer.from(newRole)],
      program.programId
    );
    const [assignmentPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("assignment"), newRolePDA.toBuffer(), stranger.publicKey.toBuffer()],
      program.programId
    );

    // stranger is not the admin
    const fakeAdminProvider = new anchor.AnchorProvider(
      provider.connection,
      new anchor.Wallet(stranger),
      {}
    );
    const fakeProgram = new anchor.Program(program.idl, fakeAdminProvider) as Program<Rbac>;

    try {
      await fakeProgram.methods
        .grantRole()
        .accounts({
          resource: resourcePDA,
          role: rolePDA,
          assignment: assignmentPDA,
          user: stranger.publicKey,
          admin: stranger.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      assert.fail("Expected NotAdmin error");
    } catch (e: any) {
      expect(e.message).to.include("NotAdmin");
    }
  });

  it("transfers admin authority", async () => {
    const newAdmin = Keypair.generate();

    await program.methods
      .transferAdmin(newAdmin.publicKey)
      .accounts({ resource: resourcePDA, admin: admin.publicKey })
      .rpc();

    const resource = await program.account.resourceAccount.fetch(resourcePDA);
    assert.equal(resource.admin.toBase58(), newAdmin.publicKey.toBase58());
  });
});
