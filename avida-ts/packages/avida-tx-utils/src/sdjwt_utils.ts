import { SDJwtVcInstance } from "@sd-jwt/sd-jwt-vc";
import { generateSalt, digest } from "@sd-jwt/crypto-nodejs";
import { subtle, createPrivateKey, sign } from "node:crypto";

export const EdDSA = {
  alg: "EdDSA",

  async getSigner(privateKeyPEM: string) {
    const privateKey = createPrivateKey(privateKeyPEM);

    return async (data: any) => {
      const encoder = new TextEncoder();
      const signature = await sign(null, encoder.encode(data), privateKey);
      return Buffer.from(signature).toString("base64url");
    };
  },
};

export async function get_sdjwt(privatePEM: string): Promise<SDJwtVcInstance> {
  const signer = await EdDSA.getSigner(privatePEM);

  return new SDJwtVcInstance({
    signer,
    signAlg: "EdDSA",
    hasher: digest,
    hashAlg: "SHA-256",
    saltGenerator: generateSalt,
  });
}
