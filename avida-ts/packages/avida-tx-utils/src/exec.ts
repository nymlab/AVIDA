import { MsgExecuteContract } from "cosmes/client";
import { getWallet } from "./wallet";
import type {
  CosmosBaseV1beta1Coin as Coin,
  CosmosBaseAbciV1beta1TxResponse as TxResponse,
} from "cosmes/protobufs";
import { type ResultAsync } from "neverthrow";
import type { UnsignedTx } from "cosmes/wallet";
import {
  feePromise,
  broadcastTxPromise,
  pollTxPromise,
  type TxUtilError,
} from "./index";

export function contractExecTx(
  chainConfigPath: string,
  executorMnemonic: string,
  contractAddr: string,
  msg: { [k: string]: unknown },
  funds: Coin[],
): ResultAsync<TxResponse, TxUtilError> {
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

  return feePromise(storeTx, wallet)
    .andThen((fee) => broadcastTxPromise(storeTx, wallet, fee))
    .andThen((txHash) => pollTxPromise(txHash, wallet));
}
