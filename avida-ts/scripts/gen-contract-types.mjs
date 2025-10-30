// @ts-check

/**
 * This script generates the src/contracts directory from the schema files in the
<<<<<<< HEAD
 * `avida/contracts`.
 **/

import codegen from "@cosmwasm/ts-codegen";
import fs from "fs";
import { dirname, join } from "path";
import { fileURLToPath } from "url";

/**
 * @typedef Contract
 * @type {object}
 * @property {string} path - relative path to the CONTRACT_DIR contract location for schema
 * @property {string} name - name of the contract to be used in types, clients, messages
 */

/**
 * Must add git full hash here for version control
 * @type {Contract[]}
 */
const CONTRACTS = [
  {
    path: "sdjwt-verifier",
    name: "SdjwtVerifier",
  },
  {
    path: "avida_example",
    name: "RestaurantContract",
  },
];

const __dirname = dirname(fileURLToPath(import.meta.url));
const CONTRACT_DIR = join(__dirname, "../..", "contracts");
const OUT_DIR = join(
  __dirname,
  "..",
  "packages",
  "avida-common-types/src/contracts",
);

// if OUT_DIR exists, remove it
if (fs.existsSync(OUT_DIR)) fs.rmdirSync(OUT_DIR, { recursive: true });

console.log("CONTRACT_DIR", CONTRACT_DIR);
console.log("OUT_DIR", OUT_DIR);

console.log("Generating TS files from JSON schema files...");

// @ts-ignore: Strange  module importing
codegen
  .default({
    contracts: CONTRACTS.map(({ path, name }) => ({
      name,
      dir: join(CONTRACT_DIR, path, "schema"),
    })),
    outPath: OUT_DIR,
    options: {
      bundle: {
        enabled: true,
        bundleFile: "contracts.ts",
      },
      client: {
        enabled: false,
      },
      messageBuilder: {
        enabled: false,
      },
      types: {
        enabled: true,
      },
      useInterchainJS: false,
    },
  })
  .then(() => {
    // Remove baseClient.ts as we don't need it and it has unwanted dependencies
    const baseClientPath = join(OUT_DIR, "baseClient.ts");
    if (fs.existsSync(baseClientPath)) {
      fs.unlinkSync(baseClientPath);
      console.log("Removed baseClient.ts");
    }

    // Remove baseClient import from contracts.ts
    const contractsPath = join(OUT_DIR, "contracts.ts");
    if (fs.existsSync(contractsPath)) {
      let contractsContent = fs.readFileSync(contractsPath, "utf-8");
      // Remove the import line
      contractsContent = contractsContent.replace(/import \* as _\d+ from "\.\/baseClient";\n/g, "");
      // Remove the export in the namespace
      contractsContent = contractsContent.replace(/export const baseClient = \{\n\s+\.\.\._\d+\n\s+\};\n/g, "");
      fs.writeFileSync(contractsPath, contractsContent);
      console.log("Removed baseClient from contracts.ts");
    }

    console.log("Contract ts generation completed successfully!");
  });
