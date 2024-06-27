export * from "./upload_and_inst";
export * from "./wallet";
export * from "./exec";
export * from "./sdjwt_utils";
import * as base64js from "base64-js";
import type {
  CosmosTxV1beta1Fee as Fee,
  CosmosBaseAbciV1beta1TxResponse as TxResponse,
} from "cosmes/protobufs";

// returns base64 encoded stringify json as `Binary` type in cosmwasm
export function toWasmBinary<T extends object | string>(data: T): string {
  const encodedData =
    typeof data === "string"
      ? new TextEncoder().encode(data)
      : new TextEncoder().encode(JSON.stringify(data));

  return base64js.fromByteArray(encodedData);
}

import { type ResultAsync, fromPromise } from "neverthrow";
import type { MnemonicWallet, UnsignedTx } from "cosmes/wallet";

export interface TxUtilError {
  EstimateFeeError: string;
  BroadcastError: string;
  PollTxError: string;
}

export const feePromise = (
  storeTx: UnsignedTx,
  wallet: MnemonicWallet,
): ResultAsync<Fee, TxUtilError> => {
  return fromPromise(wallet.estimateFee(storeTx), (e: unknown) => {
    return { EstimateFeeError: e as string } as TxUtilError;
  });
};

export const broadcastTxPromise = (
  storeTx: UnsignedTx,
  wallet: MnemonicWallet,
  fee: Fee,
): ResultAsync<string, TxUtilError> => {
  return fromPromise(wallet.broadcastTx(storeTx, fee), (e: unknown) => {
    return { BroadcastError: e as string } as TxUtilError;
  });
};

export const pollTxPromise = (
  txHash: string,
  wallet: MnemonicWallet,
): ResultAsync<TxResponse, TxUtilError> => {
  return fromPromise(wallet.pollTx(txHash), (e: unknown) => {
    return { PollTxError: e as string } as TxUtilError;
  }).map((res) => res.txResponse as TxResponse);
};
