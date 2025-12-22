"use client";

import Link from "next/link";
import { motion } from "framer-motion";
import { AppShell, PageHeader } from "@/components/layout";
import {
  SearchInput,
  InteractiveCard,
  CardHeader,
  CardTitle,
  CardDescription,
  CardFooter,
  Badge,
  ConsensusSeal,
} from "@/components/ui";
import { mockRepositories } from "@/data/mock";
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

export default function ExplorePage() {
  return (
    <AppShell>
      <PageHeader
        title="Explore"
        description="Discover repositories across the decentralized network"
      />

      <div className={styles.content}>
        <div className={styles.searchSection}>
          <SearchInput placeholder="Search repositories, users, organizations..." />
        </div>

        <div className={styles.featured}>
          <h2 className={styles.sectionTitle}>Featured Repositories</h2>
          <div className={styles.repoGrid}>
            {mockRepositories.slice(0, 4).map((repo, index) => (
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
                      </div>
                    </CardFooter>
                  </InteractiveCard>
                </Link>
              </motion.div>
            ))}
          </div>
        </div>

        <div className={styles.trending}>
          <h2 className={styles.sectionTitle}>Trending This Week</h2>
          <div className={styles.trendingList}>
            {mockRepositories.map((repo, index) => (
              <motion.div
                key={repo.id}
                initial={{ opacity: 0, x: -10 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{
                  duration: 0.18,
                  delay: 0.2 + index * 0.03,
                  ease: [0.16, 1, 0.3, 1],
                }}
              >
                <Link href={`/${repo.owner}/${repo.name}`} className={styles.trendingItem}>
                  <span className={styles.trendingRank}>{index + 1}</span>
                  <div className={styles.trendingInfo}>
                    <span className={styles.trendingName}>
                      {repo.owner}/{repo.name}
                    </span>
                    <span className={styles.trendingDesc}>{repo.description}</span>
                  </div>
                  {repo.language && (
                    <Badge variant="muted" size="sm">
                      {repo.language}
                    </Badge>
                  )}
                </Link>
              </motion.div>
            ))}
          </div>
        </div>
      </div>
    </AppShell>
  );
}
