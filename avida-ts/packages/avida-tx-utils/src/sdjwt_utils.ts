import { SDJwtVcInstance } from "@sd-jwt/sd-jwt-vc";
import { generateSalt, digest } from "@sd-jwt/crypto-nodejs";
import { createPrivateKey, sign } from "node:crypto";

export const EdDSA = {
  alg: "EdDSA",

  getSigner(privateKeyPEM: string): (data: string) => string {
    const privateKey = createPrivateKey(privateKeyPEM);

    return (data: string) => {
      const encoder = new TextEncoder();
      const signature = sign(null, encoder.encode(data), privateKey);
      return Buffer.from(signature).toString("base64url");
    };
  },
};

export function getSdJwt(privatePEM: string): SDJwtVcInstance {
  const signer = EdDSA.getSigner(privatePEM);

  return new SDJwtVcInstance({
    signer,
    signAlg: "EdDSA",
    hasher: digest,
    hashAlg: "SHA-256",
    saltGenerator: generateSalt,
  });
}
