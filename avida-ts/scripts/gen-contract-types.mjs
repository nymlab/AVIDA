// @ts-check

/**
 * This script generates the src/contracts directory from the schema files in the
<<<<<<< HEAD
 * `avida/contracts`.
 **/

import codegen from "@cosmwasm/ts-codegen";
import fs from "fs";
=======
 * repos specified in `REPOS`. It uses `buf` to generate TS files from the proto
 * files, and then generates an `index.ts` file to re-export the generated code.
 */

import { default as codegen } from "@cosmwasm/ts-codegen";
import fs from "fs";
import { mkdirSync } from "fs";
>>>>>>> b1cef2c (Change laddr to bind to 0.0.0.0 instead of 127.0.0.1 & wip: init avida-ts)
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
<<<<<<< HEAD
  {
    path: "avida_example",
    name: "RestaurantContract",
  },
=======
>>>>>>> b1cef2c (Change laddr to bind to 0.0.0.0 instead of 127.0.0.1 & wip: init avida-ts)
];

const __dirname = dirname(fileURLToPath(import.meta.url));
const CONTRACT_DIR = join(__dirname, "../..", "contracts");
<<<<<<< HEAD
const OUT_DIR = join(
  __dirname,
  "..",
  "packages",
  "avida-common-types/src/contracts",
);

// if OUT_DIR exists, remove it
if (fs.existsSync(OUT_DIR)) fs.rmdirSync(OUT_DIR, { recursive: true });
=======
const OUT_DIR = join(__dirname, "..", "packages", "avida-common-types");
>>>>>>> b1cef2c (Change laddr to bind to 0.0.0.0 instead of 127.0.0.1 & wip: init avida-ts)

console.log("CONTRACT_DIR", CONTRACT_DIR);
console.log("OUT_DIR", OUT_DIR);

console.log("Generating TS files from JSON schema files...");

<<<<<<< HEAD
// @ts-ignore: Strange  module importing
codegen
  .default({
=======
{
  await codegen.default({
>>>>>>> b1cef2c (Change laddr to bind to 0.0.0.0 instead of 127.0.0.1 & wip: init avida-ts)
    contracts: CONTRACTS.map(({ path, name }) => ({
      name,
      dir: join(CONTRACT_DIR, path),
    })),
    outPath: OUT_DIR,
    options: {
      bundle: {
<<<<<<< HEAD
        enabled: true,
        bundleFile: "contracts.ts",
      },
      client: {
        enabled: false,
      },
      messageBuilder: {
        enabled: false,
      },
    },
  })
  .then(() => {
    console.log("Contract ts generation completed successfully!");
  });
=======
        enabled: false,
      },
      client: {
        noImplicitOverride: true,
      },
      messageBuilder: {
        enabled: true,
      },
    },
  });
}

console.log("Organising output files");
{
  const generationTypes = ["type", "message"];
  const files = fs.readdirSync(OUT_DIR).filter((f) => f != ".tmp");

  // create directories for types and messages
  generationTypes.map((t) => mkdirSync(join(OUT_DIR, t.concat("s"))));

  for (const file of files) {
    if (file.includes("types")) {
      fs.renameSync(join(OUT_DIR, file), join(OUT_DIR, "types", file));
    }
    if (file.includes("message-builder")) updateMessageFileType(file);
    // TODO: remove, we are not using cosmJS clients
    if (file.includes("client")) fs.rmSync(join(OUT_DIR, file));
  }

  // Create index.ts files in  messages/, types/
  for (const type of generationTypes) {
    if (type === "type") {
      let data = CONTRACTS.map(
        ({ name }) => `import * as ${name + "Types"} from "./${name}.${type}s"`,
      ).join("\n");

      data = data.concat(
        `\nexport { ${CONTRACTS.map(({ name }) => name + "Types").join(", ")} }`,
      );
      fs.writeFileSync(join(OUT_DIR, `${type}s/index.ts`), data);
    } else {
      const data = CONTRACTS.map(
        ({ name }) => `export * from "./${name}.${type}"`,
      ).join("\n");
      fs.writeFileSync(join(OUT_DIR, `${type}s/index.ts`), data);
    }
  }

  // Create index.ts for all
  fs.writeFileSync(
    join(OUT_DIR, `index.ts`),
    `export * from "./types"\nexport * from "./messages"`,
  );
}

/**
 * @param {string} file: the name of the file
 */
function updateMessageFileType(file) {
  const subdir = "messages";
  const [name] = file.split(".");
  const fileName = file;
  const newFileName = fileName.replace("message-builder", "message");
  fs.renameSync(join(OUT_DIR, file), join(OUT_DIR, subdir, newFileName));

  // tscodegen assumes x.types.ts is in the same directory as the message-builder and clients
  // However, this is not the case when we group them together
  const fileData = fs.readFileSync(join(OUT_DIR, subdir, newFileName), "utf-8");
  const updatedFileData = fileData.replace(
    `./${name}.types`,
    `../types/${name}.types`,
  );

  fs.writeFileSync(join(OUT_DIR, subdir, newFileName), updatedFileData);
}

console.log("Contract ts generation completed successfully!");
>>>>>>> b1cef2c (Change laddr to bind to 0.0.0.0 instead of 127.0.0.1 & wip: init avida-ts)
