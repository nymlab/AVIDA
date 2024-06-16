import { MsgExecuteContract } from "cosmes/client";
import {
  CosmosBaseV1beta1Coin as Coin,
  CosmosTxV1beta1Fee as Fee,
  CosmosBaseAbciV1beta1GasInfo as GasInfo,
} from "cosmes/protobufs";
import { getWallet } from "./wallet";

import { type UnsignedTx } from "cosmes/wallet";

export async function contractExecTx(
  chainConfigPath: string,
  executorMnemonic: string,
  contractAddr: string,
  msg: { [k: string]: unknown },
  funds: Coin[],
): Promise<void> {
  console.info(
    "Executing contract at: ",
    contractAddr,
    "\n to chain specified at:",
    chainConfigPath,
  );

  const wallet = getWallet(chainConfigPath, executorMnemonic);
  console.info("Executor Addr:", wallet.address); // prints the bech32 address

  const execMsg = new MsgExecuteContract({
    sender: wallet.address,
    contract: contractAddr,
    msg,
    funds,
  });

  const storeTx: UnsignedTx = {
    msgs: [execMsg],
  };

  let fee = await wallet.estimateFee(storeTx);
  console.info("Tx fee:", fee);

  let txHash = await wallet.broadcastTx(storeTx, fee);
  console.info("Tx hash:", txHash);

  let { txResponse } = await wallet.pollTx(txHash);
  console.info("Tx response:", JSON.stringify(txResponse.events));
}
