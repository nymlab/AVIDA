import { MnemonicWallet } from "cosmes/wallet";
import fs from "fs";

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

export function getWallet(
  chainConfigPath: string,
  deployerMnemonic: string,
): MnemonicWallet {
  const chainConfig = JSON.parse(
    fs.readFileSync(chainConfigPath, "utf-8"),
  ) as ChainConfig;

  // Example usage for Osmosis chain
  return new MnemonicWallet({
    mnemonic: deployerMnemonic,
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
}
