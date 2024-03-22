import * as ed from "@noble/ed25519";

(async () => {
  const privKey =
    "2ca7546b0fbb9b7738845e60a9be01ae05fc4462ec52b7dc3dd90b88f89b5297";
  console.log("Private key:", privKey);
  const message = "abcd";
  const pubKey = await ed.getPublicKeyAsync(privKey);
  console.log("Public key:", Buffer.from(pubKey).toString("hex"));
  const signature = await ed.signAsync(message, privKey);
  console.log("Signature:", Buffer.from(signature).toString("hex"));
  const isValid = await ed.verifyAsync(signature, message, pubKey);
  console.log("Signature is valid:", isValid ? "yes" : "no");
})();
