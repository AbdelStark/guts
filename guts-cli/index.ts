import * as ed from "@noble/ed25519";

// PRIV KEY: 0x2ca7546b0fbb9b7738845e60a9be01ae05fc4462ec52b7dc3dd90b88f89b5297
const privKey = ed.utils.randomPrivateKey(); // Secure random private key

// PUB KEY: 0x1e6c5b385880849f46716d691b8a447d7cbe4a7ef154f3e2174ffb3c5256fcfe
const pubKey = await ed.getPublicKeyAsync(privKey);

async function signGitCommitLikeMessage() {
  // Simulate a simplified commit object
  const commitMessage = "Initial commit\n\nFake commit.";
  const author = "Alice <alice@example.com>";
  const committer = "Alice <alice@example.com>";
  const commitDate = "Thu Feb 24 18:04:33 2022 -0800"; // Use actual date in real scenarios
  const treeHash = "4b825dc642cb6eb9a060e54bf8d69288fbee4904"; // Simulated tree hash
  // Normally, you would include parent commit hash(es) if any
  const commitObject = `commit ${commitMessage.length}\0${commitMessage}author ${author} ${commitDate}\ncommitter ${committer} ${commitDate}\ntree ${treeHash}\n`;
  console.log("**** Commit Object ****");
  console.log(commitObject);
  console.log("************************");

  // Convert commit object to hex string
  const commitObjectHex = Buffer.from(commitObject).toString("hex");
  console.log("Commit Object Hex:", commitObjectHex);
  const pubKey = await ed.getPublicKeyAsync(privKey);
  const signature = await ed.signAsync(commitObjectHex, privKey);
  console.log("Signature:", Buffer.from(signature).toString("hex"));

  const isValid = await ed.verifyAsync(signature, commitObjectHex, pubKey);
  console.log("Signature is valid:", isValid ? "yes" : "no");
}

async function simpleSign() {
  console.log("Public key:", Buffer.from(pubKey).toString("hex"));
  console.log("Private key:", Buffer.from(privKey).toString("hex"));
  const message = "01020304abcdefaa";
  console.log("Message:", message);
  const signature = await ed.signAsync(message, privKey);
  console.log("Signature:", Buffer.from(signature).toString("hex"));
  const isValid = await ed.verifyAsync(signature, message, pubKey);
  console.log("Signature is valid:", isValid ? "yes" : "no");
}

simpleSign();

//signGitCommitLikeMessage();
