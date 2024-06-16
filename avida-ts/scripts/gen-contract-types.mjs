// @ts-check

/**
 * This script generates the src/contracts directory from the schema files in the
 * repos specified in `REPOS`. It uses `buf` to generate TS files from the proto
 * files, and then generates an `index.ts` file to re-export the generated code.
 */

import { default as codegen } from "@cosmwasm/ts-codegen";
import fs from "fs";
import { mkdirSync } from "fs";
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
const OUT_DIR = join(__dirname, "..", "packages", "avida-common-types/src");

// if OUT_DIR exists, remove it
if (fs.existsSync(OUT_DIR)) fs.rmdirSync(OUT_DIR, { recursive: true });

console.log("CONTRACT_DIR", CONTRACT_DIR);
console.log("OUT_DIR", OUT_DIR);

console.log("Generating TS files from JSON schema files...");

{
  await codegen.default({
    contracts: CONTRACTS.map(({ path, name }) => ({
      name,
      dir: join(CONTRACT_DIR, path),
    })),
    outPath: OUT_DIR,
    options: {
      bundle: {
        enabled: true,
        bundleFile: "index.ts",
      },
      client: {
        enabled: false,
      },
      messageBuilder: {
        enabled: false,
      },
    },
  });
}

//console.log("Organising output files");

//{
//  const generationTypes = ["type", "message"];
//  const files = fs.readdirSync(OUT_DIR).filter((f) => f != ".tmp");
//  console.log("files to organise", files);
//
//  // create directories for types and messages
//  generationTypes.map((t) => {
//    // if it exists, remove it
//    const outpath = join(OUT_DIR, t.concat("s"));
//    if (fs.existsSync(outpath)) {
//      fs.rmdirSync(outpath, { recursive: true });
//    }
//
//    mkdirSync(outpath);
//  });
//
//  for (const file of files) {
//    // if file has name types and not a directory, move it to types/
//    if (
//      file.includes("types") &&
//      !fs.lstatSync(join(OUT_DIR, file)).isDirectory()
//    ) {
//      fs.renameSync(join(OUT_DIR, file), join(OUT_DIR, "types", file));
//    }
//    if (file.includes("message-builder")) updateMessageFileType(file);
//    // TODO: remove, we are not using cosmJS clients
//    if (file.includes("client")) fs.rmSync(join(OUT_DIR, file));
//  }
//
//  // Create index.ts files in  messages/, types/
//  for (const type of generationTypes) {
//    if (type === "type") {
//      let data = CONTRACTS.map(
//        ({ name }) => `import * as ${name + "Types"} from "./${name}.${type}s"`,
//      ).join("\n");
//
//      data = data.concat(
//        `\nexport { ${CONTRACTS.map(({ name }) => name + "Types").join(", ")} }`,
//      );
//      fs.writeFileSync(join(OUT_DIR, `${type}s/index.ts`), data);
//    } else {
//      const data = CONTRACTS.map(
//        ({ name }) => `export * from "./${name}.${type}"`,
//      ).join("\n");
//      fs.writeFileSync(join(OUT_DIR, `${type}s/index.ts`), data);
//    }
//  }
//
//  // Create index.ts for all
//  fs.writeFileSync(
//    join(OUT_DIR, `index.ts`),
//    `export * from "./types"\nexport * from "./messages"`,
//  );
//}
//
///**
// * @param {string} file: the name of the file
// */
//function updateMessageFileType(file) {
//  const subdir = "messages";
//  const [name] = file.split(".");
//  const fileName = file;
//  const newFileName = fileName.replace("message-builder", "message");
//  fs.renameSync(join(OUT_DIR, file), join(OUT_DIR, subdir, newFileName));
//
//  // tscodegen assumes x.types.ts is in the same directory as the message-builder and clients
//  // However, this is not the case when we group them together
//  const fileData = fs.readFileSync(join(OUT_DIR, subdir, newFileName), "utf-8");
//  const updatedFileData = fileData.replace(
//    `./${name}.types`,
//    `../types/${name}.types`,
//  );
//
//  fs.writeFileSync(join(OUT_DIR, subdir, newFileName), updatedFileData);
//}

console.log("Contract ts generation completed successfully!");
