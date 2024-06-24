import { contractExecTx, local_deploy, get_sdjwt } from "@avida-ts/deployer";
import { dirname, join } from "path";
import { fileURLToPath } from "url";
import { contracts } from "@avida-ts/types";
import { utf8 } from "cosmes/codec";
import fs from "fs";
import * as base64js from "base64-js";
import { Console } from "console";

// This is from https://github.com/neutron-org/neutron/blob/main/network/init.sh
// We use this in our docker/docker-compose.local.yml
// neutron1m9l358xunhhwds0568za49mzhvuxx9ux8xafx2
const NEUTRON_DEPLOYER_MNEMONIC =
  "banner spread envelope side kite person disagree path silver will brother under couch edit food venture squirrel civil budget number acquire point work mass";
const AVIDA_SDJWT_VERIFIER =
  "neutron1qt789pxhjdawdetfz4cf8ed09d9gdwlgw7c5u5h4w40lwkudme6qchyhrw";

const __dirname = dirname(fileURLToPath(import.meta.url));
const neutronChainConfig = join(
  __dirname,
  "../docker/local-chain-config/neutron.json",
);
const avidaExampleContract = join(
  __dirname,
  "../../artifacts/avida_example.wasm",
);

export const sleep = (milliseconds) => {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
};

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

// MathOperatiosn can be "equal_to", "greater_than", "less_than"
// in rust: `pub type PresentationReq = Vec<(CriterionKey, Criterion)>`
//const req = [["age", { number: [30, "equal_to"] }]];
const req = [["age", { number: [18, "greater_than"] }]];

const encoder = new TextEncoder();
/** @type {contracts.SdjwtVerifier.RouteVerificationRequirements} */
const routeVerificationRequirements = {
  // This is Binary type
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

// ========= sleep to wait for relayer ============
await sleep(30000);

// ========= create sdjwt presentation btw issue with req disclosed ==================

// Issuer Define the claims object with the user's information
const claims = {
  firstname: "John",
  lastname: "Doe",
  age: 30,
};

// Issuer Define the disclosure frame to specify which claims can be disclosed
const disclosureFrame = {
  _sd: ["firstname", "lastname", "age"],
};

//Issue a signed JWT credential with the specified claims and disclosures
//Return a Encoded SD JWT. Issuer send the credential to the holder
const privatePEM = `-----BEGIN PRIVATE KEY-----
MC4CAQAwBQYDK2VwBCIEIFu/3i9WC60gVD1RkdN04HQRq6ht0ahpFMs37i4Qqhib
-----END PRIVATE KEY-----`;

const sdjwt = await get_sdjwt(privatePEM);

const credential = await sdjwt.issue(
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
const presentation = await sdjwt.present(credential, presentationFrame);

console.log("Presentation: ", presentation);
// ========= holder present with age disclosed to example dApp ==================

/** @type {contracts.RestaurantContract.GiveMeSomeDrink} */
const drinkMsg = {
  kind: "vc_required",
  proof: base64js.fromByteArray(new TextEncoder().encode(presentation)),
};

/** @type {contracts.RestaurantContract.ExecMsg} */
const getDrinkMsg = {
  give_me_some_drink: {
    msg: drinkMsg,
  },
};

await contractExecTx(
  neutronChainConfig,
  NEUTRON_DEPLOYER_MNEMONIC,
  exampleContractAddr,
  getDrinkMsg,
  [],
);

// ========= holder present with incorrect age disclosed, should fail ==================

// Issuer Define the claims object with the user's information
const invalid_claims = {
  firstname: "John",
  lastname: "Doe2",
  age: 10,
};

const invalid_credential = await sdjwt.issue(
  {
    iss: "issuer",
    iat: new Date().getTime(),
    vct: "https://example.com",
    sub: "holder",
    ...invalid_claims,
  },
  disclosureFrame,
);

const invalid_presentation = await sdjwt.present(
  invalid_credential,
  presentationFrame,
);

/** @type {contracts.RestaurantContract.GiveMeSomeDrink} */
const drinkMsg_with_invalid_credentials = {
  kind: "vc_required",
  proof: base64js.fromByteArray(new TextEncoder().encode(invalid_presentation)),
};

try {
  await contractExecTx(
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
} catch (err) {
  console.error("Invalid presentation failed as expected: ", err);
}
