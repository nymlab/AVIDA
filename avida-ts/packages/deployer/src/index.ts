import { MnemonicWallet } from "cosmes/wallet";
import {
  MsgStoreCode,
  MsgInstantiateContract,
  MsgExecuteContract,
} from "cosmes/client";
import fs from "fs";

// This is from https://github.com/neutron-org/neutron/blob/main/network/init.sh
// We use this in our docker/docker-compose.local.yml
const DEMO_MNEMONIC_1 =
  "banner spread envelope side kite person disagree path silver will brother under couch edit food venture squirrel civil budget number acquire point work mass";

interface ChainConfig {
  value: {
    "account-prefix": string;
    "chain-id": string;
    "rpc-addr": string;
    denom: string;
    "gas-prices": string;
  };
}

export async function main(): Promise<void> {
  // Neutron chain config in ../../../../docker/local-chain-config/neutron.json
  const chainConfig = JSON.parse(
    fs.readFileSync("./docker/local-chain-config/neutron.json", "utf-8"),
  ) as ChainConfig;

  console.log("chainConfig: ", chainConfig);
  console.log("chainConfig: ", chainConfig.value["account-prefix"]);
  console.log("chainConfig: ", chainConfig.value["chain-id"]);
  console.log("chainConfig: ", chainConfig.value["rpc-addr"]);

  // Example usage for Osmosis chain
  const deployer = new MnemonicWallet({
    mnemonic: DEMO_MNEMONIC_1,
    bech32Prefix: chainConfig.value["account-prefix"],
    chainId: chainConfig.value["chain-id"],
    rpc: chainConfig.value["rpc-addr"],
    gasPrice: {
      amount: "0.0025",
      denom: chainConfig.value.denom,
    },
    coinType: 118, // optional (default: 118)
    index: 0, // optional (default: 0)
  });
  console.log("Address:", deployer.address); // prints the bech32 address

  // Sign an arbitrary message
  const { signature } = await deployer.signArbitrary("Hello from CosmES!");
  console.log("Signature:", signature);

  // Broadcast a transaction
  // const res = await wallet.broadcastTxSync( ... );
  // console.log("Tx result:", res);
}
