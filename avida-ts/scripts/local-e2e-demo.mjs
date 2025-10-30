// @ts-check

import { contractExecTx, deploy, getSdJwt, toWasmBinary } from "@avida-ts/txutils";
import { dirname, join } from "path";
import { fileURLToPath } from "url";
import * as avidaTypes from "@avida-ts/types";

// This is from https://github.com/neutron-org/neutron/blob/main/network/init.sh
// We use this in our docker/docker-compose.local.yml
// addr: neutron1m9l358xunhhwds0568za49mzhvuxx9ux8xafx2
const NEUTRON_DEPLOYER_MNEMONIC =
  "banner spread envelope side kite person disagree path silver will brother under couch edit food venture squirrel civil budget number acquire point work mass";
const AVIDA_SDJWT_VERIFIER = process.env.CONTRACT_ADDRESS;
const __dirname = dirname(fileURLToPath(import.meta.url));
const neutronChainConfig = join(__dirname, "../docker/local-chain-config/neutron.json");
const avidaExampleContract = join(__dirname, "../../artifacts/avida_example.wasm");

/** @param {number} [milliseconds] */
export const sleep = (milliseconds) => {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
};

console.log("types", avidaTypes.contracts);

// ================== First we deploy the example dApp contract ==================
// Deploy the example contract and returns contract address
// 1. MsgStoreCode - store contract code
// 2. MsgInstantiateContract - instantiate contract with init msg
/** @type {avidaTypes.contracts.SdjwtVerifier} */
const instMsg = { verifier: AVIDA_SDJWT_VERIFIER };
const deployRes = await deploy(
  neutronChainConfig,
  avidaExampleContract,
  NEUTRON_DEPLOYER_MNEMONIC,
  instMsg,
  [],
  "avida_example",
);

if (deployRes.isErr()) {
  console.error("Error deploying example contract: ", deployRes.error);
  process.exit(1);
}
const exampleContractAddr = deployRes.value;
console.info("\n\n ---> Deployed example dApp at: ", exampleContractAddr);

console.log(avidaTypes);

// ========= Then, we use the contract registration method to register route on sdjwt-verifier ==================
// Resource and collection id defined in the cheqd-resource-artifacts data
// This resource was uploaded to the cheqd node in avida/avida-ts/docker/scripts/upload-cheqd-resource.sh
const CHEQD_RESOURCE_ID = "9fbb1b86-91f8-4942-97b9-725b7714131c";
const CHEQD_COLLECTION_ID = "5rjaLzcffhGUH4nt4fyfAg";

/** @type {avidaTypes.CheqdResourceReq} */
const CHEQD_RESOURCE_REQ = {
  resourceId: CHEQD_RESOURCE_ID,
  collectionId: CHEQD_COLLECTION_ID,
};

/** @type {avidaTypes.PresentationReq} */
const req = [["age", { number: [18, "greater_than"] }]];

/** @type {avidaTypes.contracts.SdjwtVerifier} */
const routeVerificationRequirements = {
  presentation_required: toWasmBinary(req),
  issuer_source_or_data: {
    data_or_location: toWasmBinary(CHEQD_RESOURCE_REQ),
    source: "cheqd",
  },
};

/** @type {avidaTypes.contracts.SdjwtVerifier} */
const registerRequirementMsg = {
  register_requirement: {
    msg: {
      drink: {
        requirements: routeVerificationRequirements,
      },
    },
  },
};

console.info("\n\n ---> Registering route requirements on sdjwt-verifier: ", registerRequirementMsg);

const registerRes = await contractExecTx(
  neutronChainConfig,
  NEUTRON_DEPLOYER_MNEMONIC,
  exampleContractAddr,
  registerRequirementMsg,
  [],
);

registerRes.match(
  (tx) => {
    console.info("\n\n ---> Register Route successfully: ", JSON.stringify(tx.events));
  },
  (err) => {
    console.error("\n\n ---> Register Reout failed: ", err);
  },
);

// ========= sleep to wait for relayer to relay the resource from cheqd ============
console.log("\n\n ---> Sleeping for 30 seconds to wait for relayer to relay the resource from cheqd");
await sleep(30000);

//Issue a signed JWT credential with the specified claims and disclosures
//Return a Encoded SD JWT. Issuer send the credential to the holder
const privatePEM = `-----BEGIN PRIVATE KEY-----
MC4CAQAwBQYDK2VwBCIEIFu/3i9WC60gVD1RkdN04HQRq6ht0ahpFMs37i4Qqhib
-----END PRIVATE KEY-----`;
const sdjwtInstance = getSdJwt(privatePEM);
// Issuer Define the disclosure frame to specify which claims can be disclosed

/** @type {avidaTypes.contracts.SdjwtVerifier} */
const disclosureFrame = {
  _sd: ["firstname", "lastname", "age"],
};

// ========= Success case: create sdjwt presentation and issue with req disclosed ==================
// Issuer Define the claims object with the user's information
const claims = {
  firstname: "John",
  lastname: "Doe",
  age: 30,
};

console.info("\n\n --> Issuing credential with claims that satisfied example route: ", claims);

const credential = await sdjwtInstance.issue(
  {
    iss: "issuer",
    iat: new Date().getTime(),
    vct: "https://example.com",
    sub: "holder",
    ...claims,
  },
  disclosureFrame,
);

// Holder Define the presentation frame to specify which claims should be presented
// The list of presented claims must be a subset of the disclosed claims
// the presentation frame is determined by the verifier or the protocol that was agreed upon between the holder and the verifier
const presentationFrame = { age: true };

// Create a presentation using the issued credential and the presentation frame
// return a Encoded SD JWT. Holder send the presentation to the verifier
const presentation = await sdjwtInstance.present(credential, presentationFrame);

// ========= holder present with age disclosed to example dApp ==================

/** @type {avidaTypes.contracts.RestaurantContract} */
const drinkMsg = {
  kind: "vc_required",
  proof: toWasmBinary(presentation),
};

/** @type {avidaTypes.contracts.RestaurantContract} */
const getDrinkMsg = {
  give_me_some_drink: {
    msg: drinkMsg,
  },
};

const validRes = await contractExecTx(
  neutronChainConfig,
  NEUTRON_DEPLOYER_MNEMONIC,
  exampleContractAddr,
  getDrinkMsg,
  [],
);

validRes.match(
  (tx) => {
    console.info("\n\n ---> Valid presentation succeeded: ", JSON.stringify(tx.events));
  },
  (err) => {
    console.error("\n\n ---> Valid presentation failed: ", err);
  },
);

// ========= Fail case, holder present with incorrect age disclosed, should fail ==================

const invalid_claims = {
  firstname: "John",
  lastname: "Doe2",
  age: 10,
};

console.info("\n\n ---> Issuing invalid credential with AGE not satisfying example route requirements: ", claims);

const invalid_credential = await sdjwtInstance.issue(
  {
    iss: "issuer",
    iat: new Date().getTime(),
    vct: "https://example.com",
    sub: "holder",
    ...invalid_claims,
  },
  disclosureFrame,
);

const invalid_presentation = await sdjwtInstance.present(invalid_credential, presentationFrame);

/** @type {avidaTypes.contracts.RestaurantContract} */
const drinkMsg_with_invalid_credentials = {
  kind: "vc_required",
  proof: toWasmBinary(invalid_presentation),
};

const invalidRes = await contractExecTx(
  neutronChainConfig,
  NEUTRON_DEPLOYER_MNEMONIC,
  exampleContractAddr,
  {
    give_me_some_drink: {
      msg: drinkMsg_with_invalid_credentials,
    },
  },
  [],
);

if (invalidRes.isErr()) {
  console.info(
    "\n\n ---> Invalid presentation failed as expected (uncomment for error): ",
    //invalidRes.error,
  );
}
