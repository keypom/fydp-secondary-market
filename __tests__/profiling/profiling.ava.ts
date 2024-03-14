import anyTest, { TestFn } from "ava";
import { BN, NEAR, NearAccount, Worker, getNetworkFromEnv } from "near-workspaces";
import { CONTRACT_METADATA, generateKeyPairs, LARGE_GAS, WALLET_GAS } from "../utils/general";
import { DropConfig } from "../utils/types";

const test = anyTest as TestFn<{
    worker: Worker;
    accounts: Record<string, NearAccount>;
    keypomInitialBalance: NEAR;
    keypomInitialStateStaked: NEAR;
}>;


test.beforeEach(async (t) => {
    // Comment this if you want to see console logs
    //console.log = function() {}

    // Init the worker and start a Sandbox server
    const worker = await Worker.init();

    // Prepare sandbox for tests, create accounts, deploy contracts, etc.
    const root = worker.rootAccount;

    // Creating dao member accounts
    const minqi = await root.createSubAccount('minqi');
    const benji = await root.createSubAccount('benji');

    // Deploy all 3 contracts
    const keypom = await root.devDeploy(`./__tests__/ext_wasm/keypom.wasm`);
    const marketplace = await root.devDeploy(`./out/access_key_marketplace.wasm`);

    console.log(`KEYPOM: ${keypom.accountId}`);
    console.log(`MARKETPLACE: ${marketplace.accountId}`);

    // Init the dao, Keypom, and daobot contracts
    await marketplace.call(marketplace, 'new', {
        keypom_contract: keypom.accountId,
        owner_id: minqi.accountId
    })

    await keypom.call(keypom, 'new', { root_account: 'test.near', owner_id: keypom.accountId, contract_metadata: CONTRACT_METADATA });
    
    let keypomBalance = await keypom.balance();
    console.log('keypom available INITIAL: ', keypomBalance.available.toString())
    console.log('keypom staked INITIAL: ', keypomBalance.staked.toString())
    console.log('keypom stateStaked INITIAL: ', keypomBalance.stateStaked.toString())
    console.log('keypom total INITIAL: ', keypomBalance.total.toString())

    // Save state for test runs
    t.context.worker = worker;
    t.context.accounts = { root, keypom, marketplace, minqi, benji };
});

// If the environment is reused, use test.after to replace test.afterEach
test.afterEach(async t => {
    await t.context.worker.tearDown().catch(error => {
        console.log('Failed to tear down the worker:', error);
    });
});

test('Incrementing Keys with no metadata', async t => {
    const { keypom, dao, daoBot, minqi, member1, maliciousActor, member3 } = t.context.accounts;

    let numKeysVec = [1, 5, 10, 50, 100, 200, 400, 800, 100]
    let {keys, publicKeys} = await generateKeyPairs(1);
    
    // t.is(1==1, true);
});