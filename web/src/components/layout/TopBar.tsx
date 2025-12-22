"use client";

import { useState } from "react";
import { motion } from "framer-motion";
import { Avatar, Badge } from "@/components/ui";
import { CommandPalette } from "./CommandPalette";
import styles from "./TopBar.module.css";

interface TopBarProps {
  breadcrumb?: React.ReactNode;
}

export function TopBar({ breadcrumb }: TopBarProps) {
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false);

  // Keyboard shortcut for command palette
  if (typeof window !== "undefined") {
    document.addEventListener("keydown", (e) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        setCommandPaletteOpen(true);
      }
    });
  }

  return (
    <>
      <motion.header
        className={styles.topbar}
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 0.18, ease: [0.16, 1, 0.3, 1] }}
      >
        <div className={styles.left}>
          {breadcrumb || <div className={styles.placeholder} />}
        </div>

        <div className={styles.center}>
          <button
            className={styles.searchButton}
            onClick={() => setCommandPaletteOpen(true)}
          >
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
            <span className={styles.searchText}>
              Search repos, issues, peers...
            </span>
            <div className={styles.shortcut}>
              <kbd>⌘</kbd>
              <kbd>K</kbd>
            </div>
          </button>
        </div>

        <div className={styles.right}>
          <button className={styles.iconButton} aria-label="Notifications">
            <svg viewBox="0 0 20 20" fill="none">
              <path
                d="M15 7a5 5 0 10-10 0c0 5-2 6-2 6h14s-2-1-2-6zM10 17a2 2 0 01-2-2h4a2 2 0 01-2 2z"
                stroke="currentColor"
                strokeWidth="1.5"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
            </svg>
            <span className={styles.notificationDot} />
          </button>

          <div className={styles.identity}>
            <div className={styles.identityInfo}>
              <span className={styles.identityName}>satoshi</span>
              <Badge variant="cipher" size="sm">
                7f3a...e9c2
              </Badge>
            </div>
            <Avatar fallback="S" size="sm" />
          </div>
        </div>
      </motion.header>

      <CommandPalette
        open={commandPaletteOpen}
        onOpenChange={setCommandPaletteOpen}
      />
    </>
  );
}
