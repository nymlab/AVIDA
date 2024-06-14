import { MnemonicWallet, type UnsignedTx } from "cosmes/wallet";

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
    "client-rpc-endpoint": string;
  };
}

export async function deploy(
  chainName: string = "neutron",
  deployer_mnemonic: string,
): Promise<void> {
  const chainConfig = JSON.parse(
    fs.readFileSync(`./docker/local-chain-config/${chainName}.json`, "utf-8"),
  ) as ChainConfig;

  console.log("Chain config:", chainConfig);

  // Example usage for Osmosis chain
  const deployer = new MnemonicWallet({
    mnemonic: deployer_mnemonic,
    bech32Prefix: chainConfig.value["account-prefix"],
    chainId: chainConfig.value["chain-id"],
    rpc: chainConfig.value["client-rpc-endpoint"],
    gasPrice: {
      amount: "0.0025",
      denom: chainConfig.value.denom,
    },
    coinType: 118, // optional (default: 118)
    index: 0, // optional (default: 0)
  });
  console.log("Deployer Addr:", deployer.address); // prints the bech32 address

  // Store wasm MsgStoreCode
  const wasm: Buffer = fs.readFileSync(
    "../artifacts/avida_sdjwt_verifier-aarch64.wasm",
  );
  const wasmByteCode = new Uint8Array(wasm.buffer);

  console.log("length of wasmByteCode:", wasmByteCode.length);

  const base64 = Buffer.from(wasmByteCode).toString("base64");

  console.log("length of wasm base64 string:", base64.length);

  const msg = new MsgStoreCode({
    wasmByteCode,
    sender: deployer.address,
  });

  const tx: UnsignedTx = {
    msgs: [msg],
    memo: "AVIDA: store code",
  };

  const fee = await deployer.estimateFee(tx, 0.5);
  console.log("Tx fee:", fee);

  const txHash = await deployer.broadcastTx(tx, fee);
  console.log("Tx hash:", txHash);

  const { txResponse } = await deployer.pollTx(txHash);
  console.log("Tx response:", txResponse);
}
