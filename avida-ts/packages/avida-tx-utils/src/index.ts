export * from "./upload_and_inst";
export * from "./wallet";
export * from "./exec";
export * from "./sdjwt_utils";
import * as base64js from "base64-js";

// returns base64 encoded stringify json as `Binary` type in cosmwasm
export function toWasmBinary<T extends object | string>(data: T): string {
  const encodedData =
    typeof data === "string"
      ? new TextEncoder().encode(data)
      : new TextEncoder().encode(JSON.stringify(data));

  return base64js.fromByteArray(encodedData);
}
