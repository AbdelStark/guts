"use client";

import { motion } from "framer-motion";
import { AppShell, PageHeader } from "@/components/layout";
import {
  Badge,
  ConsensusSeal,
  Avatar,
  Tabs,
  TabsList,
  TabsTrigger,
  TabsContent,
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  TableCell,
} from "@/components/ui";
import { mockPullRequests } from "@/data/mock";
import styles from "./page.module.css";

export default function PullRequestsPage() {
  const openPRs = mockPullRequests.filter((pr) => pr.state === "open");
  const closedPRs = mockPullRequests.filter((pr) => pr.state !== "open");

  return (
    <AppShell>
      <PageHeader
        title="Pull Requests"
        description="Code changes awaiting review and consensus"
      />

      <div className={styles.content}>
        <Tabs defaultValue="open">
          <TabsList>
            <TabsTrigger value="open" count={openPRs.length}>
              Open
            </TabsTrigger>
            <TabsTrigger value="closed" count={closedPRs.length}>
              Closed
            </TabsTrigger>
          </TabsList>

          <TabsContent value="open">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Pull Request</TableHead>
                  <TableHead>Author</TableHead>
                  <TableHead>Branches</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Updated</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {openPRs.map((pr, index) => (
                  <motion.tr
                    key={pr.id}
                    initial={{ opacity: 0, y: 6 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{
                      duration: 0.18,
                      delay: index * 0.03,
                      ease: [0.16, 1, 0.3, 1],
                    }}
                    className={styles.row}
                  >
                    <TableCell>
                      <div className={styles.prInfo}>
                        <span className={styles.prNumber}>#{pr.number}</span>
                        <span className={styles.prTitle}>{pr.title}</span>
                        <div className={styles.labels}>
                          {pr.labels.map((label) => (
                            <Badge key={label} variant="muted" size="sm">
                              {label}
                            </Badge>
                          ))}
                        </div>
                      </div>
                    </TableCell>
                    <TableCell>
                      <div className={styles.author}>
                        <Avatar fallback={pr.author[0].toUpperCase()} size="sm" />
                        <span>{pr.author}</span>
                      </div>
                    </TableCell>
                    <TableCell>
                      <div className={styles.branches}>
                        <Badge variant="cipher" size="sm">
                          {pr.sourceBranch}
                        </Badge>
                        <span className={styles.arrow}>→</span>
                        <Badge variant="muted" size="sm">
                          {pr.targetBranch}
                        </Badge>
                      </div>
                    </TableCell>
                    <TableCell>
                      <ConsensusSeal
                        status={pr.consensusStatus}
                        quorumSize={7}
                        nodeCount={12}
                      />
                    </TableCell>
                    <TableCell>
                      <span className={styles.time}>{pr.updatedAt}</span>
                    </TableCell>
                  </motion.tr>
                ))}
              </TableBody>
            </Table>
          </TabsContent>

          <TabsContent value="closed">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Pull Request</TableHead>
                  <TableHead>Author</TableHead>
                  <TableHead>State</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Updated</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {closedPRs.map((pr, index) => (
                  <motion.tr
                    key={pr.id}
                    initial={{ opacity: 0, y: 6 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{
                      duration: 0.18,
                      delay: index * 0.03,
                      ease: [0.16, 1, 0.3, 1],
                    }}
                    className={styles.row}
                  >
                    <TableCell>
                      <div className={styles.prInfo}>
                        <span className={styles.prNumber}>#{pr.number}</span>
                        <span className={styles.prTitle}>{pr.title}</span>
                      </div>
                    </TableCell>
                    <TableCell>
                      <div className={styles.author}>
                        <Avatar fallback={pr.author[0].toUpperCase()} size="sm" />
                        <span>{pr.author}</span>
                      </div>
                    </TableCell>
                    <TableCell>
                      <Badge
                        variant={pr.state === "merged" ? "success" : "muted"}
                        size="sm"
                      >
                        {pr.state}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      <ConsensusSeal
                        status={pr.consensusStatus}
                        quorumSize={7}
                        nodeCount={12}
                      />
                    </TableCell>
                    <TableCell>
                      <span className={styles.time}>{pr.updatedAt}</span>
                    </TableCell>
                  </motion.tr>
                ))}
              </TableBody>
            </Table>
          </TabsContent>
        </Tabs>
      </div>
    </AppShell>
  );
}
