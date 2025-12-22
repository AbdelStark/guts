"use client";

import { motion } from "framer-motion";
import { AppShell, PageHeader } from "@/components/layout";
import {
  Card,
  CardHeader,
  CardTitle,
  CardContent,
  Badge,
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  TableCell,
} from "@/components/ui";
import { mockNodes } from "@/data/mock";
import styles from "./page.module.css";

export default function NodesPage() {
  const onlineNodes = mockNodes.filter((n) => n.status === "online").length;
  const validators = mockNodes.filter((n) => n.consensusRole === "validator").length;
  const totalPeers = mockNodes.reduce((acc, n) => acc + n.peers, 0);

  const getStatusVariant = (status: string) => {
    switch (status) {
      case "online":
        return "success";
      case "syncing":
        return "warning";
      case "offline":
        return "danger";
      default:
        return "muted";
    }
  };

  return (
    <AppShell>
      <PageHeader
        title="Network Nodes"
        description="View the decentralized network of GUTS validators and observers"
      />

      <div className={styles.content}>
        <div className={styles.stats}>
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.18, ease: [0.16, 1, 0.3, 1] }}
          >
            <Card>
              <CardContent>
                <div className={styles.statCard}>
                  <div className={styles.statIcon}>
                    <svg viewBox="0 0 24 24" fill="none">
                      <circle cx="12" cy="6" r="3" stroke="currentColor" strokeWidth="2" />
                      <circle cx="6" cy="18" r="3" stroke="currentColor" strokeWidth="2" />
                      <circle cx="18" cy="18" r="3" stroke="currentColor" strokeWidth="2" />
                      <path
                        d="M12 9v3M9 15l-2 2M15 15l2 2"
                        stroke="currentColor"
                        strokeWidth="2"
                        strokeLinecap="round"
                      />
                    </svg>
                  </div>
                  <div className={styles.statInfo}>
                    <span className={styles.statValue}>{mockNodes.length}</span>
                    <span className={styles.statLabel}>Total Nodes</span>
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
              <CardContent>
                <div className={styles.statCard}>
                  <div className={styles.statIconOnline}>
                    <svg viewBox="0 0 24 24" fill="none">
                      <circle cx="12" cy="12" r="8" stroke="currentColor" strokeWidth="2" />
                      <path
                        d="M9 12l2 2 4-4"
                        stroke="currentColor"
                        strokeWidth="2"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                      />
                    </svg>
                  </div>
                  <div className={styles.statInfo}>
                    <span className={styles.statValue}>{onlineNodes}</span>
                    <span className={styles.statLabel}>Online</span>
                  </div>
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
              <CardContent>
                <div className={styles.statCard}>
                  <div className={styles.statIconValidator}>
                    <svg viewBox="0 0 24 24" fill="none">
                      <path
                        d="M12 2L4 6v6c0 5.5 3.5 10.5 8 12 4.5-1.5 8-6.5 8-12V6l-8-4z"
                        stroke="currentColor"
                        strokeWidth="2"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                      />
                      <path
                        d="M9 12l2 2 4-4"
                        stroke="currentColor"
                        strokeWidth="2"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                      />
                    </svg>
                  </div>
                  <div className={styles.statInfo}>
                    <span className={styles.statValue}>{validators}</span>
                    <span className={styles.statLabel}>Validators</span>
                  </div>
                </div>
              </CardContent>
            </Card>
          </motion.div>

          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.18, delay: 0.12, ease: [0.16, 1, 0.3, 1] }}
          >
            <Card>
              <CardContent>
                <div className={styles.statCard}>
                  <div className={styles.statIcon}>
                    <svg viewBox="0 0 24 24" fill="none">
                      <path
                        d="M4 12h4M16 12h4M12 4v4M12 16v4"
                        stroke="currentColor"
                        strokeWidth="2"
                        strokeLinecap="round"
                      />
                      <circle cx="12" cy="12" r="3" stroke="currentColor" strokeWidth="2" />
                    </svg>
                  </div>
                  <div className={styles.statInfo}>
                    <span className={styles.statValue}>{totalPeers}</span>
                    <span className={styles.statLabel}>Peer Connections</span>
                  </div>
                </div>
              </CardContent>
            </Card>
          </motion.div>
        </div>

        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.18, delay: 0.16, ease: [0.16, 1, 0.3, 1] }}
        >
          <Card padding="none">
            <CardHeader>
              <CardTitle>Network Nodes</CardTitle>
            </CardHeader>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Public Key</TableHead>
                  <TableHead>Address</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Role</TableHead>
                  <TableHead>Peers</TableHead>
                  <TableHead>Location</TableHead>
                  <TableHead>Last Seen</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {mockNodes.map((node, index) => (
                  <motion.tr
                    key={node.id}
                    initial={{ opacity: 0, y: 6 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{
                      duration: 0.18,
                      delay: 0.2 + index * 0.03,
                      ease: [0.16, 1, 0.3, 1],
                    }}
                    className={styles.row}
                  >
                    <TableCell mono>
                      {node.publicKey.slice(0, 8)}...{node.publicKey.slice(-8)}
                    </TableCell>
                    <TableCell mono>{node.address}</TableCell>
                    <TableCell>
                      <Badge variant={getStatusVariant(node.status)} size="sm" dot>
                        {node.status}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      <Badge
                        variant={node.consensusRole === "validator" ? "cipher" : "muted"}
                        size="sm"
                      >
                        {node.consensusRole}
                      </Badge>
                    </TableCell>
                    <TableCell>{node.peers}</TableCell>
                    <TableCell>{node.location || "Unknown"}</TableCell>
                    <TableCell>
                      <span className={styles.time}>{node.lastSeen}</span>
                    </TableCell>
                  </motion.tr>
                ))}
              </TableBody>
            </Table>
          </Card>
        </motion.div>
      </div>
    </AppShell>
  );
}
