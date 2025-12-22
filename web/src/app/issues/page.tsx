"use client";

import { motion } from "framer-motion";
import { AppShell, PageHeader } from "@/components/layout";
import {
  Button,
  Badge,
  Avatar,
  AvatarGroup,
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
import { mockIssues } from "@/data/mock";
import styles from "./page.module.css";

export default function IssuesPage() {
  const openIssues = mockIssues.filter((issue) => issue.state === "open");
  const closedIssues = mockIssues.filter((issue) => issue.state === "closed");

  const getLabelVariant = (label: string) => {
    if (label === "bug" || label === "critical") return "danger";
    if (label === "enhancement") return "success";
    if (label === "security") return "warning";
    return "muted";
  };

  return (
    <AppShell>
      <PageHeader
        title="Issues"
        description="Track bugs, features, and discussions"
        actions={
          <Button
            leftIcon={
              <svg viewBox="0 0 16 16" fill="none">
                <path
                  d="M8 3v10M3 8h10"
                  stroke="currentColor"
                  strokeWidth="1.5"
                  strokeLinecap="round"
                />
              </svg>
            }
          >
            New Issue
          </Button>
        }
      />

      <div className={styles.content}>
        <Tabs defaultValue="open">
          <TabsList>
            <TabsTrigger value="open" count={openIssues.length}>
              Open
            </TabsTrigger>
            <TabsTrigger value="closed" count={closedIssues.length}>
              Closed
            </TabsTrigger>
          </TabsList>

          <TabsContent value="open">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Issue</TableHead>
                  <TableHead>Author</TableHead>
                  <TableHead>Assignees</TableHead>
                  <TableHead>Comments</TableHead>
                  <TableHead>Updated</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {openIssues.map((issue, index) => (
                  <motion.tr
                    key={issue.id}
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
                      <div className={styles.issueInfo}>
                        <div className={styles.issueHeader}>
                          <span className={styles.issueIcon}>
                            <svg viewBox="0 0 16 16" fill="none">
                              <circle
                                cx="8"
                                cy="8"
                                r="6"
                                stroke="currentColor"
                                strokeWidth="1.5"
                              />
                              <circle cx="8" cy="8" r="2" fill="currentColor" />
                            </svg>
                          </span>
                          <span className={styles.issueNumber}>#{issue.number}</span>
                          <span className={styles.issueTitle}>{issue.title}</span>
                        </div>
                        <div className={styles.labels}>
                          {issue.labels.map((label) => (
                            <Badge
                              key={label}
                              variant={getLabelVariant(label)}
                              size="sm"
                            >
                              {label}
                            </Badge>
                          ))}
                        </div>
                      </div>
                    </TableCell>
                    <TableCell>
                      <div className={styles.author}>
                        <Avatar fallback={issue.author[0].toUpperCase()} size="sm" />
                        <span>{issue.author}</span>
                      </div>
                    </TableCell>
                    <TableCell>
                      {issue.assignees.length > 0 ? (
                        <AvatarGroup max={3}>
                          {issue.assignees.map((assignee) => (
                            <Avatar
                              key={assignee}
                              fallback={assignee[0].toUpperCase()}
                              size="sm"
                            />
                          ))}
                        </AvatarGroup>
                      ) : (
                        <span className={styles.noAssignees}>No assignees</span>
                      )}
                    </TableCell>
                    <TableCell>
                      <div className={styles.comments}>
                        <svg viewBox="0 0 16 16" fill="none">
                          <path
                            d="M14 10c0 .55-.45 1-1 1H5l-3 3V3c0-.55.45-1 1-1h10c.55 0 1 .45 1 1v7z"
                            stroke="currentColor"
                            strokeWidth="1.5"
                          />
                        </svg>
                        <span>{issue.comments}</span>
                      </div>
                    </TableCell>
                    <TableCell>
                      <span className={styles.time}>{issue.updatedAt}</span>
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
                  <TableHead>Issue</TableHead>
                  <TableHead>Author</TableHead>
                  <TableHead>Closed</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {closedIssues.map((issue, index) => (
                  <motion.tr
                    key={issue.id}
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
                      <div className={styles.issueInfo}>
                        <div className={styles.issueHeader}>
                          <span className={styles.issueIconClosed}>
                            <svg viewBox="0 0 16 16" fill="none">
                              <circle
                                cx="8"
                                cy="8"
                                r="6"
                                stroke="currentColor"
                                strokeWidth="1.5"
                              />
                              <path
                                d="M6 8l1.5 1.5L10 6"
                                stroke="currentColor"
                                strokeWidth="1.5"
                                strokeLinecap="round"
                                strokeLinejoin="round"
                              />
                            </svg>
                          </span>
                          <span className={styles.issueNumber}>#{issue.number}</span>
                          <span className={styles.issueTitleClosed}>{issue.title}</span>
                        </div>
                      </div>
                    </TableCell>
                    <TableCell>
                      <div className={styles.author}>
                        <Avatar fallback={issue.author[0].toUpperCase()} size="sm" />
                        <span>{issue.author}</span>
                      </div>
                    </TableCell>
                    <TableCell>
                      <span className={styles.time}>{issue.updatedAt}</span>
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
