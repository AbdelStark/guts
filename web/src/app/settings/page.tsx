"use client";

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
  Input,
  Badge,
} from "@/components/ui";
import styles from "./page.module.css";

export default function SettingsPage() {
  return (
    <AppShell>
      <PageHeader
        title="Settings"
        description="Configure your GUTS client and preferences"
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
                <CardTitle>Network Configuration</CardTitle>
                <CardDescription>
                  Configure how your node connects to the GUTS network.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className={styles.formGroup}>
                  <Input
                    label="Bootstrap Peers"
                    defaultValue="135.181.42.156:9000, 95.216.89.23:9000"
                    hint="Comma-separated list of initial peers to connect to"
                  />
                </div>
                <div className={styles.formGroup}>
                  <Input
                    label="Listen Address"
                    defaultValue="0.0.0.0:9000"
                    hint="Address and port for incoming P2P connections"
                  />
                </div>
                <div className={styles.formGroup}>
                  <Input
                    label="API Address"
                    defaultValue="127.0.0.1:8080"
                    hint="Address for the local HTTP API"
                  />
                </div>
              </CardContent>
              <CardFooter>
                <Button variant="secondary">Reset to Defaults</Button>
                <Button>Save Changes</Button>
              </CardFooter>
            </Card>
          </motion.div>

          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.18, delay: 0.04, ease: [0.16, 1, 0.3, 1] }}
          >
            <Card>
              <CardHeader>
                <CardTitle>Appearance</CardTitle>
                <CardDescription>
                  Customize the look and feel of GUTS.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className={styles.settingRow}>
                  <div className={styles.settingInfo}>
                    <span className={styles.settingLabel}>Theme</span>
                    <span className={styles.settingDesc}>
                      Choose your preferred color scheme
                    </span>
                  </div>
                  <div className={styles.themeButtons}>
                    <button className={styles.themeButtonActive}>
                      <span className={styles.themeDark} />
                      Dark
                    </button>
                    <button className={styles.themeButton}>
                      <span className={styles.themeLight} />
                      Light
                    </button>
                    <button className={styles.themeButton}>
                      <span className={styles.themeSystem} />
                      System
                    </button>
                  </div>
                </div>
                <div className={styles.settingRow}>
                  <div className={styles.settingInfo}>
                    <span className={styles.settingLabel}>Reduced Motion</span>
                    <span className={styles.settingDesc}>
                      Minimize animations for accessibility
                    </span>
                  </div>
                  <Badge variant="muted" size="sm">
                    System
                  </Badge>
                </div>
              </CardContent>
            </Card>
          </motion.div>

          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.18, delay: 0.08, ease: [0.16, 1, 0.3, 1] }}
          >
            <Card>
              <CardHeader>
                <CardTitle>About</CardTitle>
                <CardDescription>
                  Information about your GUTS installation.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className={styles.aboutGrid}>
                  <div className={styles.aboutItem}>
                    <span className={styles.aboutLabel}>Version</span>
                    <span className={styles.aboutValue}>0.1.0-alpha</span>
                  </div>
                  <div className={styles.aboutItem}>
                    <span className={styles.aboutLabel}>Commit</span>
                    <span className={styles.aboutValueMono}>ce50cc3</span>
                  </div>
                  <div className={styles.aboutItem}>
                    <span className={styles.aboutLabel}>Build Date</span>
                    <span className={styles.aboutValue}>Dec 22, 2025</span>
                  </div>
                  <div className={styles.aboutItem}>
                    <span className={styles.aboutLabel}>Rust Version</span>
                    <span className={styles.aboutValueMono}>1.83.0</span>
                  </div>
                </div>
              </CardContent>
              <CardFooter>
                <Button variant="secondary">
                  <svg viewBox="0 0 16 16" fill="none" width={16} height={16}>
                    <path
                      d="M8 1a7 7 0 00-2.21 13.64c.35.07.48-.15.48-.33v-1.3c-1.96.42-2.37-.84-2.37-.84-.32-.81-.78-1.03-.78-1.03-.64-.44.05-.43.05-.43.7.05 1.07.72 1.07.72.63 1.08 1.65.77 2.05.59.06-.46.25-.77.45-.95-1.57-.18-3.22-.78-3.22-3.5 0-.77.28-1.4.73-1.9-.07-.18-.32-.9.07-1.87 0 0 .6-.19 1.95.72a6.75 6.75 0 013.56 0c1.36-.91 1.95-.72 1.95-.72.39.97.14 1.69.07 1.87.45.5.73 1.13.73 1.9 0 2.72-1.65 3.32-3.23 3.5.25.22.48.65.48 1.3v1.93c0 .19.12.41.48.34A7 7 0 008 1z"
                      fill="currentColor"
                    />
                  </svg>
                  View on GitHub
                </Button>
                <Button variant="secondary">Check for Updates</Button>
              </CardFooter>
            </Card>
          </motion.div>
        </div>
      </div>
    </AppShell>
  );
}
