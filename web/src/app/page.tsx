"use client";

import Link from "next/link";
import { motion } from "framer-motion";
import { AppShell, PageHeader } from "@/components/layout";
import {
  Button,
  InteractiveCard,
  CardHeader,
  CardTitle,
  CardDescription,
  CardFooter,
  Badge,
  ConsensusSeal,
  Avatar,
  Tabs,
  TabsList,
  TabsTrigger,
  TabsContent,
} from "@/components/ui";
import { mockRepositories, mockActivity } from "@/data/mock";
import styles from "./page.module.css";

const languageColors: Record<string, string> = {
  Rust: "#dea584",
  TypeScript: "#3178c6",
  JavaScript: "#f1e05a",
  Go: "#00add8",
  Python: "#3572a5",
  MDX: "#fcb32c",
  LaTeX: "#008080",
};

export default function Dashboard() {
  return (
    <AppShell>
      <PageHeader
        title="Repositories"
        description="Your code, sovereign and decentralized"
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
            New Repository
          </Button>
        }
      />

      <div className={styles.content}>
        <div className={styles.main}>
          <Tabs defaultValue="all">
            <TabsList>
              <TabsTrigger value="all" count={mockRepositories.length}>
                All
              </TabsTrigger>
              <TabsTrigger
                value="public"
                count={mockRepositories.filter((r) => r.visibility === "public").length}
              >
                Public
              </TabsTrigger>
              <TabsTrigger
                value="private"
                count={mockRepositories.filter((r) => r.visibility === "private").length}
              >
                Private
              </TabsTrigger>
            </TabsList>

            <TabsContent value="all">
              <div className={styles.repoGrid}>
                {mockRepositories.map((repo, index) => (
                  <motion.div
                    key={repo.id}
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{
                      duration: 0.18,
                      delay: index * 0.04,
                      ease: [0.16, 1, 0.3, 1],
                    }}
                  >
                    <Link href={`/${repo.owner}/${repo.name}`} className={styles.repoLink}>
                      <InteractiveCard padding="md">
                        <CardHeader
                          actions={
                            <ConsensusSeal
                              status={repo.consensusStatus}
                              quorumSize={7}
                              nodeCount={12}
                              timestamp="2 min ago"
                            />
                          }
                        >
                          <CardTitle>
                            <span className={styles.repoOwner}>{repo.owner}</span>
                            <span className={styles.repoSeparator}>/</span>
                            <span className={styles.repoName}>{repo.name}</span>
                          </CardTitle>
                          <CardDescription>{repo.description}</CardDescription>
                        </CardHeader>

                        <CardFooter>
                          <div className={styles.repoMeta}>
                            {repo.language && (
                              <div className={styles.language}>
                                <span
                                  className={styles.languageDot}
                                  style={{
                                    background: languageColors[repo.language] || "#888",
                                  }}
                                />
                                {repo.language}
                              </div>
                            )}
                            <Badge variant="muted" size="sm">
                              {repo.visibility}
                            </Badge>
                          </div>
                          <div className={styles.repoStats}>
                            <span>{repo.stats.commits} commits</span>
                            <span>{repo.stats.issues} issues</span>
                          </div>
                        </CardFooter>
                      </InteractiveCard>
                    </Link>
                  </motion.div>
                ))}
              </div>
            </TabsContent>

            <TabsContent value="public">
              <div className={styles.repoGrid}>
                {mockRepositories
                  .filter((r) => r.visibility === "public")
                  .map((repo, index) => (
                    <motion.div
                      key={repo.id}
                      initial={{ opacity: 0, y: 10 }}
                      animate={{ opacity: 1, y: 0 }}
                      transition={{
                        duration: 0.18,
                        delay: index * 0.04,
                        ease: [0.16, 1, 0.3, 1],
                      }}
                    >
                      <Link href={`/${repo.owner}/${repo.name}`} className={styles.repoLink}>
                        <InteractiveCard padding="md">
                          <CardHeader
                            actions={
                              <ConsensusSeal
                                status={repo.consensusStatus}
                                quorumSize={7}
                                nodeCount={12}
                                timestamp="2 min ago"
                              />
                            }
                          >
                            <CardTitle>
                              <span className={styles.repoOwner}>{repo.owner}</span>
                              <span className={styles.repoSeparator}>/</span>
                              <span className={styles.repoName}>{repo.name}</span>
                            </CardTitle>
                            <CardDescription>{repo.description}</CardDescription>
                          </CardHeader>

                          <CardFooter>
                            <div className={styles.repoMeta}>
                              {repo.language && (
                                <div className={styles.language}>
                                  <span
                                    className={styles.languageDot}
                                    style={{
                                      background: languageColors[repo.language] || "#888",
                                    }}
                                  />
                                  {repo.language}
                                </div>
                              )}
                            </div>
                            <div className={styles.repoStats}>
                              <span>{repo.stats.commits} commits</span>
                              <span>{repo.stats.issues} issues</span>
                            </div>
                          </CardFooter>
                        </InteractiveCard>
                      </Link>
                    </motion.div>
                  ))}
              </div>
            </TabsContent>

            <TabsContent value="private">
              <div className={styles.repoGrid}>
                {mockRepositories
                  .filter((r) => r.visibility === "private")
                  .map((repo, index) => (
                    <motion.div
                      key={repo.id}
                      initial={{ opacity: 0, y: 10 }}
                      animate={{ opacity: 1, y: 0 }}
                      transition={{
                        duration: 0.18,
                        delay: index * 0.04,
                        ease: [0.16, 1, 0.3, 1],
                      }}
                    >
                      <Link href={`/${repo.owner}/${repo.name}`} className={styles.repoLink}>
                        <InteractiveCard padding="md">
                          <CardHeader
                            actions={
                              <ConsensusSeal
                                status={repo.consensusStatus}
                                quorumSize={7}
                                nodeCount={12}
                                timestamp="2 min ago"
                              />
                            }
                          >
                            <CardTitle>
                              <span className={styles.repoOwner}>{repo.owner}</span>
                              <span className={styles.repoSeparator}>/</span>
                              <span className={styles.repoName}>{repo.name}</span>
                            </CardTitle>
                            <CardDescription>{repo.description}</CardDescription>
                          </CardHeader>

                          <CardFooter>
                            <div className={styles.repoMeta}>
                              {repo.language && (
                                <div className={styles.language}>
                                  <span
                                    className={styles.languageDot}
                                    style={{
                                      background: languageColors[repo.language] || "#888",
                                    }}
                                  />
                                  {repo.language}
                                </div>
                              )}
                              <Badge variant="muted" size="sm">
                                private
                              </Badge>
                            </div>
                            <div className={styles.repoStats}>
                              <span>{repo.stats.commits} commits</span>
                              <span>{repo.stats.issues} issues</span>
                            </div>
                          </CardFooter>
                        </InteractiveCard>
                      </Link>
                    </motion.div>
                  ))}
              </div>
            </TabsContent>
          </Tabs>
        </div>

        <aside className={styles.sidebar}>
          <div className={styles.activitySection}>
            <h2 className={styles.sectionTitle}>Recent Activity</h2>
            <div className={styles.activityList}>
              {mockActivity.map((activity, index) => (
                <motion.div
                  key={activity.id}
                  className={styles.activityItem}
                  initial={{ opacity: 0, x: 10 }}
                  animate={{ opacity: 1, x: 0 }}
                  transition={{
                    duration: 0.18,
                    delay: 0.1 + index * 0.03,
                    ease: [0.16, 1, 0.3, 1],
                  }}
                >
                  <Avatar fallback={activity.author[0].toUpperCase()} size="sm" />
                  <div className={styles.activityContent}>
                    <p className={styles.activityTitle}>{activity.title}</p>
                    <p className={styles.activityMeta}>
                      {activity.author} · {activity.timestamp}
                    </p>
                  </div>
                </motion.div>
              ))}
            </div>
          </div>
        </aside>
      </div>
    </AppShell>
  );
}
