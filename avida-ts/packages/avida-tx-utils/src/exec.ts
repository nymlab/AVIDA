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
): Promise<any> {
  console.info(
    "Executing contract at: ",
    contractAddr,
    "\n to chain specified at:",
    chainConfigPath,
  );

  const wallet = getWallet(chainConfigPath, executorMnemonic);

  const execMsg = new MsgExecuteContract({
    sender: wallet.address,
    contract: contractAddr,
    msg,
    funds,
  });
  console.info("\t Executing Msg: ", JSON.stringify(execMsg), "\n\n");

  const storeTx: UnsignedTx = {
    msgs: [execMsg],
  };

  let fee = await wallet.estimateFee(storeTx);
  let txHash = await wallet.broadcastTx(storeTx, fee);
  let { txResponse } = await wallet.pollTx(txHash);
  console.info(
    "\t Execute Msg response:",
    JSON.stringify(txResponse.events),
    "\n\n",
  );

  return txResponse;
}
