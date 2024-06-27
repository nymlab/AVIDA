import { MsgStoreCode, MsgInstantiateContract } from "cosmes/client";
import {
  CosmosBaseV1beta1Coin as Coin,
  CosmosTxV1beta1Fee as Fee,
  CosmosBaseAbciV1beta1GasInfo as GasInfo,
} from "cosmes/protobufs";

import { getWallet } from "./wallet";
import fs from "fs";

import { type UnsignedTx } from "cosmes/wallet";

export async function local_deploy(
  chainConfigPath: string,
  contractPath: string,
  deployerMnemonic: string,
  contractInstMsg: { [k: string]: unknown },
  initFund: Coin[],
  contractLabel: string,
  memo: string | undefined = undefined,
): Promise<string> {
  console.info(
    "Deploying contract at: ",
    contractPath,
    "\n to chain specified at:",
    chainConfigPath,
  );

  const deployer = getWallet(chainConfigPath, deployerMnemonic);
  console.info("Deployer Addr:", deployer.address); // prints the bech32 address

  // Store wasm MsgStoreCode
  const wasm: Buffer = fs.readFileSync(contractPath);
  const wasmByteCode = new Uint8Array(wasm.buffer);

  console.debug("length of wasmByteCode:", wasmByteCode.length);

  const storeMsg = new MsgStoreCode({
    wasmByteCode,
    sender: deployer.address,
  });

  const storeTx: UnsignedTx = {
    msgs: [storeMsg],
    memo,
  };

  let fee = await deployer.estimateFee(storeTx);
  console.info("Tx fee:", fee);

  let txHash = await deployer.broadcastTx(storeTx, fee);
  console.info("Tx hash:", txHash);

  let { txResponse: storeRes } = await deployer.pollTx(txHash);
  console.info("Tx response:", JSON.stringify(storeRes.events));

  // find the codeId from the events which is in the format:
  // {"type":"store_code","attributes":[{"key":"code_checksum","value":"8d4fb9c2161cf3f3df81a9f401b0540f33bbd70e61a1bb58c45dca6c1a1f772e","index":true},{"key":"code_id","value":"22","index":true}
  const codeIdBigInt = BigInt(
    storeRes.events
      .find((e) => e.type === "store_code")
      ?.attributes.find((a) => a.key === "code_id")?.value ?? 0n,
  );
  if (codeIdBigInt === 0n) {
    throw new Error("Code ID not found in tx events");
  }

  console.info("code id:", codeIdBigInt.toString(10));

  console.log(contractInstMsg);
  const instMsg = new MsgInstantiateContract({
    sender: deployer.address,
    admin: deployer.address,
    codeId: codeIdBigInt,
    msg: contractInstMsg,
    funds: initFund,
    label: contractLabel,
  });

  const instTx: UnsignedTx = {
    msgs: [instMsg],
    memo,
  };

  fee = await deployer.estimateFee(instTx);
  console.info("Tx fee:", fee);

  txHash = await deployer.broadcastTx(instTx, fee);
  console.info("Tx hash:", txHash);

  let { txResponse: instRes } = await deployer.pollTx(txHash);

  const contractAddr =
    instRes.events
      .find((e) => e.type === "instantiate")
      ?.attributes.find((a) => a.key === "_contract_address")?.value ??
    undefined;

  if (!contractAddr) {
    console.error("Contract address not found in the tx events");
    throw new Error("Contract address not found in the tx events");
  }

  return contractAddr;
}
