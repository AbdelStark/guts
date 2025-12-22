"use client";

import { useState, useEffect, useCallback } from "react";
import { useRouter } from "next/navigation";
import * as DialogPrimitive from "@radix-ui/react-dialog";
import { motion, AnimatePresence } from "framer-motion";
import clsx from "clsx";
import styles from "./CommandPalette.module.css";

interface CommandPaletteProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

interface CommandItem {
  id: string;
  label: string;
  description?: string;
  icon: React.ReactNode;
  action: () => void;
  category: "navigation" | "repos" | "actions";
}

const navigationCommands: CommandItem[] = [
  {
    id: "go-repos",
    label: "Go to Repositories",
    icon: (
      <svg viewBox="0 0 20 20" fill="none">
        <path
          d="M3 5a2 2 0 012-2h4.586a1 1 0 01.707.293l1.414 1.414a1 1 0 00.707.293H15a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2V5z"
          stroke="currentColor"
          strokeWidth="1.5"
        />
      </svg>
    ),
    action: () => {},
    category: "navigation",
  },
  {
    id: "go-pulls",
    label: "Go to Pull Requests",
    icon: (
      <svg viewBox="0 0 20 20" fill="none">
        <path
          d="M5 3v14M5 6a3 3 0 106 0 3 3 0 00-6 0zM15 17a3 3 0 100-6 3 3 0 000 6zM15 11V8a2 2 0 00-2-2H9"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
        />
      </svg>
    ),
    action: () => {},
    category: "navigation",
  },
  {
    id: "go-issues",
    label: "Go to Issues",
    icon: (
      <svg viewBox="0 0 20 20" fill="none">
        <circle cx="10" cy="10" r="7" stroke="currentColor" strokeWidth="1.5" />
        <circle cx="10" cy="10" r="2" fill="currentColor" />
      </svg>
    ),
    action: () => {},
    category: "navigation",
  },
  {
    id: "go-nodes",
    label: "Go to Nodes",
    icon: (
      <svg viewBox="0 0 20 20" fill="none">
        <circle cx="10" cy="5" r="2" stroke="currentColor" strokeWidth="1.5" />
        <circle cx="5" cy="15" r="2" stroke="currentColor" strokeWidth="1.5" />
        <circle cx="15" cy="15" r="2" stroke="currentColor" strokeWidth="1.5" />
        <path
          d="M10 7v3M8.5 11.5L6 13.5M11.5 11.5L14 13.5"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
        />
        <circle cx="10" cy="12" r="2" stroke="currentColor" strokeWidth="1.5" />
      </svg>
    ),
    action: () => {},
    category: "navigation",
  },
  {
    id: "go-identity",
    label: "Go to Identity",
    icon: (
      <svg viewBox="0 0 20 20" fill="none">
        <path
          d="M10 3L2 7l8 4 8-4-8-4z"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
        <path
          d="M2 13l8 4 8-4M2 10l8 4 8-4"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      </svg>
    ),
    action: () => {},
    category: "navigation",
  },
];

const repoCommands: CommandItem[] = [
  {
    id: "repo-guts",
    label: "guts-org/guts",
    description: "Decentralized code collaboration platform",
    icon: (
      <svg viewBox="0 0 20 20" fill="none">
        <path
          d="M3 5a2 2 0 012-2h4.586a1 1 0 01.707.293l1.414 1.414a1 1 0 00.707.293H15a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2V5z"
          stroke="currentColor"
          strokeWidth="1.5"
        />
      </svg>
    ),
    action: () => {},
    category: "repos",
  },
  {
    id: "repo-sdk",
    label: "guts-org/sdk",
    description: "Client SDK for GUTS network",
    icon: (
      <svg viewBox="0 0 20 20" fill="none">
        <path
          d="M3 5a2 2 0 012-2h4.586a1 1 0 01.707.293l1.414 1.414a1 1 0 00.707.293H15a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2V5z"
          stroke="currentColor"
          strokeWidth="1.5"
        />
      </svg>
    ),
    action: () => {},
    category: "repos",
  },
];

const actionCommands: CommandItem[] = [
  {
    id: "create-repo",
    label: "Create new repository",
    icon: (
      <svg viewBox="0 0 20 20" fill="none">
        <path
          d="M10 5v10M5 10h10"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
        />
      </svg>
    ),
    action: () => {},
    category: "actions",
  },
  {
    id: "create-issue",
    label: "Create new issue",
    icon: (
      <svg viewBox="0 0 20 20" fill="none">
        <path
          d="M10 5v10M5 10h10"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
        />
      </svg>
    ),
    action: () => {},
    category: "actions",
  },
];

export function CommandPalette({ open, onOpenChange }: CommandPaletteProps) {
  const [query, setQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const router = useRouter();

  const allCommands = [...navigationCommands, ...repoCommands, ...actionCommands];

  const filteredCommands = query
    ? allCommands.filter(
        (cmd) =>
          cmd.label.toLowerCase().includes(query.toLowerCase()) ||
          cmd.description?.toLowerCase().includes(query.toLowerCase())
      )
    : allCommands;

  const groupedCommands = {
    navigation: filteredCommands.filter((c) => c.category === "navigation"),
    repos: filteredCommands.filter((c) => c.category === "repos"),
    actions: filteredCommands.filter((c) => c.category === "actions"),
  };

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          setSelectedIndex((i) =>
            i < filteredCommands.length - 1 ? i + 1 : 0
          );
          break;
        case "ArrowUp":
          e.preventDefault();
          setSelectedIndex((i) =>
            i > 0 ? i - 1 : filteredCommands.length - 1
          );
          break;
        case "Enter":
          e.preventDefault();
          const selected = filteredCommands[selectedIndex];
          if (selected) {
            if (selected.id.startsWith("go-")) {
              const path = selected.id.replace("go-", "/").replace("repos", "");
              router.push(path === "/repos" ? "/" : path);
            } else if (selected.id.startsWith("repo-")) {
              router.push(`/${selected.label}`);
            }
            onOpenChange(false);
          }
          break;
        case "Escape":
          onOpenChange(false);
          break;
      }
    },
    [filteredCommands, selectedIndex, router, onOpenChange]
  );

  useEffect(() => {
    setSelectedIndex(0);
  }, [query]);

  useEffect(() => {
    if (!open) {
      setQuery("");
      setSelectedIndex(0);
    }
  }, [open]);

  let currentIndex = 0;

  return (
    <AnimatePresence>
      {open && (
        <DialogPrimitive.Root open={open} onOpenChange={onOpenChange}>
          <DialogPrimitive.Portal>
            <DialogPrimitive.Overlay asChild>
              <motion.div
                className={styles.overlay}
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                transition={{ duration: 0.15 }}
              />
            </DialogPrimitive.Overlay>
            <DialogPrimitive.Content asChild>
              <motion.div
                className={styles.content}
                initial={{ opacity: 0, scale: 0.96, y: -20 }}
                animate={{ opacity: 1, scale: 1, y: 0 }}
                exit={{ opacity: 0, scale: 0.96, y: -20 }}
                transition={{ duration: 0.15, ease: [0.16, 1, 0.3, 1] }}
                onKeyDown={handleKeyDown}
              >
                <div className={styles.inputWrapper}>
                  <svg
                    className={styles.searchIcon}
                    viewBox="0 0 20 20"
                    fill="none"
                  >
                    <path
                      d="M17.5 17.5L13.875 13.875M15.8333 9.16667C15.8333 12.8486 12.8486 15.8333 9.16667 15.8333C5.48477 15.8333 2.5 12.8486 2.5 9.16667C2.5 5.48477 5.48477 2.5 9.16667 2.5C12.8486 2.5 15.8333 5.48477 15.8333 9.16667Z"
                      stroke="currentColor"
                      strokeWidth="1.5"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                    />
                  </svg>
                  <input
                    className={styles.input}
                    placeholder="Search repos, issues, peers..."
                    value={query}
                    onChange={(e) => setQuery(e.target.value)}
                    autoFocus
                  />
                  <div className={styles.shortcut}>
                    <kbd>esc</kbd>
                  </div>
                </div>

                <div className={styles.results}>
                  {groupedCommands.navigation.length > 0 && (
                    <div className={styles.group}>
                      <div className={styles.groupLabel}>Navigation</div>
                      {groupedCommands.navigation.map((cmd) => {
                        const index = currentIndex++;
                        return (
                          <button
                            key={cmd.id}
                            className={clsx(
                              styles.item,
                              selectedIndex === index && styles.selected
                            )}
                            onClick={() => {
                              const path = cmd.id.replace("go-", "/").replace("repos", "");
                              router.push(path === "/repos" ? "/" : path);
                              onOpenChange(false);
                            }}
                            onMouseEnter={() => setSelectedIndex(index)}
                          >
                            <span className={styles.itemIcon}>{cmd.icon}</span>
                            <span className={styles.itemLabel}>{cmd.label}</span>
                          </button>
                        );
                      })}
                    </div>
                  )}

                  {groupedCommands.repos.length > 0 && (
                    <div className={styles.group}>
                      <div className={styles.groupLabel}>Repositories</div>
                      {groupedCommands.repos.map((cmd) => {
                        const index = currentIndex++;
                        return (
                          <button
                            key={cmd.id}
                            className={clsx(
                              styles.item,
                              selectedIndex === index && styles.selected
                            )}
                            onClick={() => {
                              router.push(`/${cmd.label}`);
                              onOpenChange(false);
                            }}
                            onMouseEnter={() => setSelectedIndex(index)}
                          >
                            <span className={styles.itemIcon}>{cmd.icon}</span>
                            <div className={styles.itemContent}>
                              <span className={styles.itemLabel}>{cmd.label}</span>
                              {cmd.description && (
                                <span className={styles.itemDescription}>
                                  {cmd.description}
                                </span>
                              )}
                            </div>
                          </button>
                        );
                      })}
                    </div>
                  )}

                  {groupedCommands.actions.length > 0 && (
                    <div className={styles.group}>
                      <div className={styles.groupLabel}>Actions</div>
                      {groupedCommands.actions.map((cmd) => {
                        const index = currentIndex++;
                        return (
                          <button
                            key={cmd.id}
                            className={clsx(
                              styles.item,
                              selectedIndex === index && styles.selected
                            )}
                            onClick={() => {
                              cmd.action();
                              onOpenChange(false);
                            }}
                            onMouseEnter={() => setSelectedIndex(index)}
                          >
                            <span className={styles.itemIcon}>{cmd.icon}</span>
                            <span className={styles.itemLabel}>{cmd.label}</span>
                          </button>
                        );
                      })}
                    </div>
                  )}

                  {filteredCommands.length === 0 && (
                    <div className={styles.empty}>
                      <span>No results found for &ldquo;{query}&rdquo;</span>
                    </div>
                  )}
                </div>

                <div className={styles.footer}>
                  <div className={styles.footerHint}>
                    <kbd>↑↓</kbd>
                    <span>Navigate</span>
                  </div>
                  <div className={styles.footerHint}>
                    <kbd>↵</kbd>
                    <span>Select</span>
                  </div>
                  <div className={styles.footerHint}>
                    <kbd>esc</kbd>
                    <span>Close</span>
                  </div>
                </div>
              </motion.div>
            </DialogPrimitive.Content>
          </DialogPrimitive.Portal>
        </DialogPrimitive.Root>
      )}
    </AnimatePresence>
  );
}
