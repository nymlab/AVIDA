import { MsgExecuteContract } from "cosmes/client";
import { getWallet } from "./wallet";
import type {
  CosmosBaseV1beta1Coin as Coin,
  CosmosBaseAbciV1beta1TxResponse as TxResponse,
} from "cosmes/protobufs";
import { type UnsignedTx } from "cosmes/wallet";

export async function contractExecTx(
  chainConfigPath: string,
  executorMnemonic: string,
  contractAddr: string,
  msg: { [k: string]: unknown },
  funds: Coin[],
): Promise<TxResponse> {
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

  const fee = await wallet.estimateFee(storeTx);
  const txHash = await wallet.broadcastTx(storeTx, fee);
  const { txResponse } = await wallet.pollTx(txHash);
  console.info(
    "\t Execute Msg response:",
    JSON.stringify(txResponse.events),
    "\n\n",
  );

  return txResponse as TxResponse;
}
