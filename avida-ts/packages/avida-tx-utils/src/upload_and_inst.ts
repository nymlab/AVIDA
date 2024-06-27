import { MsgStoreCode, MsgInstantiateContract } from "cosmes/client";
import type { CosmosBaseV1beta1Coin as Coin } from "cosmes/protobufs";
import { getWallet } from "./wallet";
import fs from "fs";
import { type Result, ok, err } from "neverthrow";

import { type UnsignedTx } from "cosmes/wallet";
import {
  feePromise,
  broadcastTxPromise,
  pollTxPromise,
  type TxUtilError,
} from "./index";

export interface DeployError {
  StoreCodeError: string;
  InstantiateError: string;
}

export async function deploy(
  chainConfigPath: string,
  contractPath: string,
  deployerMnemonic: string,
  contractInstMsg: { [k: string]: unknown },
  initFund: Coin[],
  contractLabel: string,
  memo: string | undefined = undefined,
): Promise<Result<string, TxUtilError | DeployError>> {
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

  const storeRes = await feePromise(storeTx, deployer)
    .andThen((fee) => broadcastTxPromise(storeTx, deployer, fee))
    .andThen((txHash) => pollTxPromise(txHash, deployer));
  if (storeRes.isErr()) {
    return err(storeRes.error);
  }
  // find the codeId from the events which is in the format:
  // {"type":"store_code","attributes":[{"key":"code_checksum","value":"8d4fb9c2161cf3f3df81a9f401b0540f33bbd70e61a1bb58c45dca6c1a1f772e","index":true},{"key":"code_id","value":"22","index":true}
  const codeIdBigInt = BigInt(
    storeRes.value.events
      .find((e) => e.type === "store_code")
      ?.attributes.find((a) => a.key === "code_id")?.value ?? 0n,
  );
  if (codeIdBigInt === 0n) {
    return err({
      StoreCodeError: "Code ID not found in the tx events",
    } as DeployError);
  }
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

  const instRes = await feePromise(instTx, deployer)
    .andThen((fee) => broadcastTxPromise(instTx, deployer, fee))
    .andThen((txHash) => pollTxPromise(txHash, deployer));
  if (instRes.isErr()) {
    return err(instRes.error);
  }

  const contractAddr =
    instRes.value.events
      .find((e) => e.type === "instantiate")
      ?.attributes.find((a) => a.key === "_contract_address")?.value ??
    undefined;

  if (!contractAddr) {
    return err({
      InstantiateError: "Contract address not found in the tx events",
    } as DeployError);
  }

  return ok(contractAddr);
}
