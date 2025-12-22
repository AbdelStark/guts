"use client";

import { useState } from "react";
import { motion } from "framer-motion";
import { AppShell, PageHeader } from "@/components/layout";
import {
  Button,
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardFooter,
  Badge,
  Input,
} from "@/components/ui";
import styles from "./page.module.css";

export default function IdentityPage() {
  const [showPrivateKey, setShowPrivateKey] = useState(false);

  const identity = {
    publicKey: "7f3a9b2c4e5d6f8a1b3c5e7d9f1a3b5c7e9d1f3a5b7c9e1d3f5a7b9c1e3d5f7a9b",
    fingerprint: "7f3a...9b2c",
    createdAt: "2024-01-15T10:30:00Z",
    algorithm: "Ed25519",
  };

  return (
    <AppShell>
      <PageHeader
        title="Identity"
        description="Your sovereign cryptographic identity on the GUTS network"
      />

      <div className={styles.content}>
        <div className={styles.grid}>
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.18, ease: [0.16, 1, 0.3, 1] }}
          >
            <Card>
              <CardHeader>
                <CardTitle>Public Key</CardTitle>
                <CardDescription>
                  Your identity on the GUTS network. Share this with others.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className={styles.keyDisplay}>
                  <code className={styles.keyCode}>{identity.publicKey}</code>
                  <Button variant="secondary" size="sm">
                    <svg viewBox="0 0 16 16" fill="none" width={14} height={14}>
                      <rect
                        x="4"
                        y="4"
                        width="8"
                        height="8"
                        rx="1"
                        stroke="currentColor"
                        strokeWidth="1.5"
                      />
                      <path
                        d="M4 12V3a1 1 0 011-1h8"
                        stroke="currentColor"
                        strokeWidth="1.5"
                        strokeLinecap="round"
                      />
                    </svg>
                    Copy
                  </Button>
                </div>
                <div className={styles.keyMeta}>
                  <div className={styles.keyMetaItem}>
                    <span className={styles.keyMetaLabel}>Fingerprint</span>
                    <Badge variant="cipher">{identity.fingerprint}</Badge>
                  </div>
                  <div className={styles.keyMetaItem}>
                    <span className={styles.keyMetaLabel}>Algorithm</span>
                    <span className={styles.keyMetaValue}>{identity.algorithm}</span>
                  </div>
                </div>
              </CardContent>
            </Card>
          </motion.div>

          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.18, delay: 0.04, ease: [0.16, 1, 0.3, 1] }}
          >
            <Card>
              <CardHeader>
                <CardTitle>Profile</CardTitle>
                <CardDescription>
                  Optional display name for your identity.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <Input
                  label="Display Name"
                  defaultValue="satoshi"
                  hint="This name is stored locally and shared when you interact with repos."
                />
              </CardContent>
              <CardFooter>
                <Button variant="secondary">Cancel</Button>
                <Button>Save Changes</Button>
              </CardFooter>
            </Card>
          </motion.div>

          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.18, delay: 0.08, ease: [0.16, 1, 0.3, 1] }}
          >
            <Card>
              <CardHeader>
                <CardTitle>Export Identity</CardTitle>
                <CardDescription>
                  Back up your identity to use on another device.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className={styles.warning}>
                  <svg viewBox="0 0 20 20" fill="none">
                    <path
                      d="M10 3L2 17h16L10 3z"
                      stroke="currentColor"
                      strokeWidth="1.5"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                    />
                    <path
                      d="M10 8v3M10 14h.01"
                      stroke="currentColor"
                      strokeWidth="1.5"
                      strokeLinecap="round"
                    />
                  </svg>
                  <div>
                    <p className={styles.warningTitle}>Keep your private key safe</p>
                    <p className={styles.warningText}>
                      Anyone with access to your private key can impersonate you on the
                      network. Never share it with anyone.
                    </p>
                  </div>
                </div>
                {showPrivateKey ? (
                  <div className={styles.privateKeyDisplay}>
                    <code className={styles.privateKey}>
                      ************************************************************
                    </code>
                    <Button
                      variant="secondary"
                      size="sm"
                      onClick={() => setShowPrivateKey(false)}
                    >
                      Hide
                    </Button>
                  </div>
                ) : (
                  <Button
                    variant="secondary"
                    onClick={() => setShowPrivateKey(true)}
                  >
                    <svg viewBox="0 0 16 16" fill="none" width={16} height={16}>
                      <path
                        d="M1 8s2.5-5 7-5 7 5 7 5-2.5 5-7 5-7-5-7-5z"
                        stroke="currentColor"
                        strokeWidth="1.5"
                      />
                      <circle cx="8" cy="8" r="2" stroke="currentColor" strokeWidth="1.5" />
                    </svg>
                    Reveal Private Key
                  </Button>
                )}
              </CardContent>
            </Card>
          </motion.div>

          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.18, delay: 0.12, ease: [0.16, 1, 0.3, 1] }}
          >
            <Card>
              <CardHeader>
                <CardTitle>Danger Zone</CardTitle>
                <CardDescription>
                  Irreversible actions that affect your identity.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className={styles.dangerActions}>
                  <div className={styles.dangerItem}>
                    <div>
                      <p className={styles.dangerTitle}>Rotate Identity</p>
                      <p className={styles.dangerText}>
                        Generate a new key pair. Your old identity will be invalidated.
                      </p>
                    </div>
                    <Button variant="destructive">Rotate</Button>
                  </div>
                  <div className={styles.dangerItem}>
                    <div>
                      <p className={styles.dangerTitle}>Delete Identity</p>
                      <p className={styles.dangerText}>
                        Permanently delete your identity from this device.
                      </p>
                    </div>
                    <Button variant="destructive">Delete</Button>
                  </div>
                </div>
              </CardContent>
            </Card>
          </motion.div>
        </div>
      </div>
    </AppShell>
  );
}
