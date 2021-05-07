
const assert = require("assert");
const anchor = require("@project-serum/anchor");
const solana = require("@solana/web3.js");

describe("Aggregator", () => {
  const provider = anchor.Provider.local();

  anchor.setProvider(provider);

  const aggregator = anchor.workspace.Aggregator;
  const node = anchor.workspace.Node;

  let nodeASigner = null;
  let nodeANonce = null;
  const nodeA = new anchor.web3.Account();


  it("Is runs the constructor", async () => {
    await aggregator.state.rpc.new({
      accounts: {
        authority: provider.wallet.publicKey,
      },
    });
    const state = await aggregator.state();
    assert.ok(state.nodeSigners[0].equals(provider.wallet.publicKey))

  });

  it("Able to add a node and write data into", async () => {
    let [
      _nodeSigner,
      _nonce,
    ] = await anchor.web3.PublicKey.findProgramAddress(
        [nodeA.publicKey.toBuffer()],
        node.programId
    );
    nodeASigner = _nodeSigner;
    nodeANonce = _nonce;

    await node.state.rpc.new(nodeANonce, {
      accounts: {
        authority: provider.wallet.publicKey,
        node: nodeA.publicKey,
        nodeSigner:nodeASigner
      },
    });

    await aggregator.state.rpc.addNode(nodeASigner, {
      accounts: {
        authority: provider.wallet.publicKey,
      },
    });
    const state = await aggregator.state();

    assert.ok(state.nodeSigners[0].equals(nodeASigner))
  });

  it("Data node for node A should write data into aggregator", async () => {
    const newData = new anchor.BN(2134)
    await node.state.rpc.writeToAggregator(newData, {
      accounts: {
        authority: provider.wallet.publicKey,
        node: nodeA.publicKey,
        nodeSigner: nodeASigner,
        cpiState: await aggregator.state.address(),
        aggregatorProgram: aggregator.programId,
      },
    })
    const state = await aggregator.state();
    const nodeState = await node.state();
    assert.ok(state.nodeData[0].eq(newData))
    assert.ok(!nodeState.lastReportEpoch.eq(new anchor.BN(0)))
  });
});
