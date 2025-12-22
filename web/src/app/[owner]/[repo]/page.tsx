"use client";

import { useParams } from "next/navigation";
import { motion } from "framer-motion";
import { AppShell, PageHeader, Breadcrumb } from "@/components/layout";
import {
  Button,
  Card,
  CardContent,
  Badge,
  ConsensusSeal,
  Tabs,
  TabsList,
  TabsTrigger,
  TabsContent,
} from "@/components/ui";
import { mockRepositories, mockCode, mockFileTree } from "@/data/mock";

import styles from "./page.module.css";

// Generate static params for all mock repositories (required for static export)
export function generateStaticParams() {
  return mockRepositories.map((repo) => ({
    owner: repo.owner,
    repo: repo.name,
  }));
}

interface FileNode {
  name: string;
  type: "file" | "directory";
  language?: string;
  children?: FileNode[];
}

function FileTree({ nodes, depth = 0 }: { nodes: FileNode[]; depth?: number }) {
  return (
    <ul className={styles.fileList} style={{ paddingLeft: depth > 0 ? 16 : 0 }}>
      {nodes.map((node) => (
        <li key={node.name} className={styles.fileItem}>
          <button className={styles.fileButton}>
            {node.type === "directory" ? (
              <svg viewBox="0 0 16 16" fill="none" className={styles.fileIcon}>
                <path
                  d="M2 4a1 1 0 011-1h3.586a1 1 0 01.707.293l1.414 1.414a1 1 0 00.707.293H13a1 1 0 011 1v6a1 1 0 01-1 1H3a1 1 0 01-1-1V4z"
                  stroke="currentColor"
                  strokeWidth="1.5"
                />
              </svg>
            ) : (
              <svg viewBox="0 0 16 16" fill="none" className={styles.fileIcon}>
                <path
                  d="M4 2h5.586a1 1 0 01.707.293l2.414 2.414a1 1 0 01.293.707V13a1 1 0 01-1 1H4a1 1 0 01-1-1V3a1 1 0 011-1z"
                  stroke="currentColor"
                  strokeWidth="1.5"
                />
              </svg>
            )}
            <span className={styles.fileName}>{node.name}</span>
          </button>
          {node.children && <FileTree nodes={node.children} depth={depth + 1} />}
        </li>
      ))}
    </ul>
  );
}

export default function RepositoryPage() {
  const params = useParams();
  const owner = params.owner as string;
  const repoName = params.repo as string;

  const repo = mockRepositories.find(
    (r) => r.owner === owner && r.name === repoName
  ) || mockRepositories[0];

  return (
    <AppShell
      breadcrumb={
        <Breadcrumb
          items={[
            { label: repo.owner, href: `/${repo.owner}` },
            { label: repo.name },
          ]}
        />
      }
    >
      <PageHeader
        title={`${repo.owner}/${repo.name}`}
        description={repo.description}
        actions={
          <div className={styles.headerActions}>
            <ConsensusSeal
              status={repo.consensusStatus}
              quorumSize={7}
              nodeCount={12}
              timestamp="2 min ago"
            />
            <Button variant="secondary">
              <svg viewBox="0 0 16 16" fill="none" width={16} height={16}>
                <path
                  d="M8 2v3M8 11v3M2 8h3M11 8h3M4.22 4.22l2.12 2.12M9.66 9.66l2.12 2.12M4.22 11.78l2.12-2.12M9.66 6.34l2.12-2.12"
                  stroke="currentColor"
                  strokeWidth="1.5"
                  strokeLinecap="round"
                />
              </svg>
              Fork
            </Button>
            <Button variant="secondary">
              <svg viewBox="0 0 16 16" fill="none" width={16} height={16}>
                <path
                  d="M8 3L6 5.5M8 3l2 2.5M8 3v8M3 10v2a1 1 0 001 1h8a1 1 0 001-1v-2"
                  stroke="currentColor"
                  strokeWidth="1.5"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
              </svg>
              Clone
            </Button>
          </div>
        }
      />

      <div className={styles.content}>
        <div className={styles.repoInfo}>
          <div className={styles.stats}>
            <div className={styles.statItem}>
              <svg viewBox="0 0 16 16" fill="none">
                <circle cx="8" cy="8" r="6" stroke="currentColor" strokeWidth="1.5" />
                <path
                  d="M8 5v3l2 1"
                  stroke="currentColor"
                  strokeWidth="1.5"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
              </svg>
              <span>{repo.stats.commits} commits</span>
            </div>
            <div className={styles.statItem}>
              <svg viewBox="0 0 16 16" fill="none">
                <path
                  d="M5 3v10M5 7h4a2 2 0 002-2V3M5 13h6a2 2 0 002-2v-1"
                  stroke="currentColor"
                  strokeWidth="1.5"
                  strokeLinecap="round"
                />
              </svg>
              <span>{repo.stats.branches} branches</span>
            </div>
            <div className={styles.statItem}>
              <svg viewBox="0 0 16 16" fill="none">
                <circle cx="8" cy="8" r="6" stroke="currentColor" strokeWidth="1.5" />
                <circle cx="8" cy="8" r="2" fill="currentColor" />
              </svg>
              <span>{repo.stats.issues} issues</span>
            </div>
            <div className={styles.statItem}>
              <svg viewBox="0 0 16 16" fill="none">
                <path
                  d="M4 3v10M4 5a2 2 0 104 0 2 2 0 00-4 0zM12 13a2 2 0 100-4 2 2 0 000 4zM12 9V7a1.5 1.5 0 00-1.5-1.5H8"
                  stroke="currentColor"
                  strokeWidth="1.5"
                  strokeLinecap="round"
                />
              </svg>
              <span>{repo.stats.pullRequests} pull requests</span>
            </div>
          </div>
          <Badge variant="muted">{repo.visibility}</Badge>
        </div>

        <Tabs defaultValue="code">
          <TabsList>
            <TabsTrigger
              value="code"
              icon={
                <svg viewBox="0 0 16 16" fill="none">
                  <path
                    d="M5.5 4.5L2 8l3.5 3.5M10.5 4.5L14 8l-3.5 3.5"
                    stroke="currentColor"
                    strokeWidth="1.5"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  />
                </svg>
              }
            >
              Code
            </TabsTrigger>
            <TabsTrigger
              value="issues"
              count={repo.stats.issues}
              icon={
                <svg viewBox="0 0 16 16" fill="none">
                  <circle cx="8" cy="8" r="6" stroke="currentColor" strokeWidth="1.5" />
                  <circle cx="8" cy="8" r="2" fill="currentColor" />
                </svg>
              }
            >
              Issues
            </TabsTrigger>
            <TabsTrigger
              value="pulls"
              count={repo.stats.pullRequests}
              icon={
                <svg viewBox="0 0 16 16" fill="none">
                  <path
                    d="M4 3v10M4 5a2 2 0 104 0 2 2 0 00-4 0zM12 13a2 2 0 100-4 2 2 0 000 4zM12 9V7a1.5 1.5 0 00-1.5-1.5H8"
                    stroke="currentColor"
                    strokeWidth="1.5"
                    strokeLinecap="round"
                  />
                </svg>
              }
            >
              Pull Requests
            </TabsTrigger>
          </TabsList>

          <TabsContent value="code">
            <motion.div
              className={styles.codeView}
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.18, ease: [0.16, 1, 0.3, 1] }}
            >
              <aside className={styles.fileTree}>
                <div className={styles.fileTreeHeader}>
                  <Badge variant="cipher" size="sm">
                    main
                  </Badge>
                </div>
                <FileTree nodes={mockFileTree as FileNode[]} />
              </aside>

              <Card className={styles.codePanel} padding="none">
                <div className={styles.codeHeader}>
                  <span className={styles.fileName}>src/consensus/simplex.rs</span>
                  <div className={styles.codeActions}>
                    <Button variant="ghost" size="sm">
                      Raw
                    </Button>
                    <Button variant="ghost" size="sm">
                      Copy
                    </Button>
                  </div>
                </div>
                <pre className={styles.code}>
                  <code>{mockCode}</code>
                </pre>
              </Card>
            </motion.div>
          </TabsContent>

          <TabsContent value="issues">
            <Card>
              <CardContent>
                <p className={styles.placeholder}>
                  Issues for this repository will appear here.
                </p>
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="pulls">
            <Card>
              <CardContent>
                <p className={styles.placeholder}>
                  Pull requests for this repository will appear here.
                </p>
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </div>
    </AppShell>
  );
}
