import { contractExecTx, local_deploy } from "@avida-ts/deployer";
import { queryContract, queryContractParams } from "cosmes/client";
import { dirname, join } from "path";
import { fileURLToPath } from "url";
import { contracts } from "@avida-ts/types";
import { utf8 } from "cosmes/codec";
import fs from "fs";
import * as base64js from "base64-js";

// This is from https://github.com/neutron-org/neutron/blob/main/network/init.sh
// We use this in our docker/docker-compose.local.yml
// neutron1m9l358xunhhwds0568za49mzhvuxx9ux8xafx2
const NEUTRON_DEPLOYER_MNEMONIC =
  "banner spread envelope side kite person disagree path silver will brother under couch edit food venture squirrel civil budget number acquire point work mass";
const AVIDA_SDJWT_VERIFIER =
  "neutron1ev8e7z53nm9ncn8jv5efsuv7k5stn7uyt627qmpk9a98netlua0qumsp9n";

const __dirname = dirname(fileURLToPath(import.meta.url));
const neutronChainConfig = join(
  __dirname,
  "../docker/local-chain-config/neutron.json",
);
const avidaExampleContract = join(
  __dirname,
  "../../artifacts/avida_example.wasm",
);

// ================== First we deploy the example dApp contract ==================
// Deploy the example contract and returns contract address
// 1. MsgStoreCode - store contract code
// 2. MsgInstantiateContract - instantiate contract with init msg
/** @type {contracts.SdjwtVerifier.InstantiateMsg} */
const instMsg = { verifier: AVIDA_SDJWT_VERIFIER };

const exampleContractAddr = await local_deploy(
  neutronChainConfig,
  avidaExampleContract,
  NEUTRON_DEPLOYER_MNEMONIC,
  instMsg,
  [],
  "avida_example",
);

console.info("Deployed example dApp at: ", exampleContractAddr);

// ========= use the contract registration method to register route on sdjwt-verifier ==================
// Resource and collection id defined in the cheqd-resource-artifacts data
const CHEQD_RESOURCE_ID = "9fbb1b86-91f8-4942-97b9-725b7714131c";
const CHEQD_COLLECTION_ID = "5rjaLzcffhGUH4nt4fyfAg";
const CHEQD_RESOURCE_REQ = {
  resourceId: CHEQD_RESOURCE_ID,
  collectionId: CHEQD_COLLECTION_ID,
};

const jwk = fs
  .readFileSync(
    join(__dirname, "../docker/cheqd-resource-artifacts/jwk.json"),
    "ascii",
  )
  .trim();

const req = [["age", { number: [30, "equal_to"] }]];

const encoder = new TextEncoder();
/** @type {contracts.SdjwtVerifier.RouteVerificationRequirements} */
const routeVerificationRequirements = {
  // This is Binary type
  // pub type PresentationReq = Vec<(CriterionKey, Criterion)>;
  presentation_request: base64js.fromByteArray(
    new TextEncoder().encode(JSON.stringify(req)),
  ),
  verification_source: {
    // Binary type
    data_or_location: base64js.fromByteArray(
      new TextEncoder().encode(JSON.stringify(CHEQD_RESOURCE_REQ)),
    ),
    source: "cheqd",
  },
};

console.log("Route verification requirements: ", routeVerificationRequirements);

/** @type {contracts.SdjwtVerifier.ExecMsg} */
const registerRequirementMsg = {
  register_requirement: {
    msg: {
      drink: {
        requirements: routeVerificationRequirements,
      },
    },
  },
};

await contractExecTx(
  neutronChainConfig,
  NEUTRON_DEPLOYER_MNEMONIC,
  exampleContractAddr,
  registerRequirementMsg,
  [],
);

// ========= create sdjwt presentation btw issue with req disclosed ==================
