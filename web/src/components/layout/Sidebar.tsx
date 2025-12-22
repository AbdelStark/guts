"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { motion } from "framer-motion";
import clsx from "clsx";
import { SyncStatus } from "@/components/ui";
import styles from "./Sidebar.module.css";

const navItems = [
  {
    label: "Repos",
    href: "/",
    icon: (
      <svg viewBox="0 0 20 20" fill="none">
        <path
          d="M3 5a2 2 0 012-2h4.586a1 1 0 01.707.293l1.414 1.414a1 1 0 00.707.293H15a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2V5z"
          stroke="currentColor"
          strokeWidth="1.5"
        />
      </svg>
    ),
  },
  {
    label: "Pull Requests",
    href: "/pulls",
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
  },
  {
    label: "Issues",
    href: "/issues",
    icon: (
      <svg viewBox="0 0 20 20" fill="none">
        <circle cx="10" cy="10" r="7" stroke="currentColor" strokeWidth="1.5" />
        <circle cx="10" cy="10" r="2" fill="currentColor" />
      </svg>
    ),
  },
  {
    label: "Explore",
    href: "/explore",
    icon: (
      <svg viewBox="0 0 20 20" fill="none">
        <path
          d="M10 10l4-2-2 4-4 2 2-4z"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
        <circle cx="10" cy="10" r="7" stroke="currentColor" strokeWidth="1.5" />
      </svg>
    ),
  },
  {
    label: "Nodes",
    href: "/nodes",
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
  },
];

const bottomItems = [
  {
    label: "Identity",
    href: "/identity",
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
  },
  {
    label: "Settings",
    href: "/settings",
    icon: (
      <svg viewBox="0 0 20 20" fill="none">
        <circle cx="10" cy="10" r="2.5" stroke="currentColor" strokeWidth="1.5" />
        <path
          d="M10 2v2M10 16v2M18 10h-2M4 10H2M15.66 4.34l-1.42 1.42M5.76 14.24l-1.42 1.42M15.66 15.66l-1.42-1.42M5.76 5.76L4.34 4.34"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
        />
      </svg>
    ),
  },
];

export function Sidebar() {
  const pathname = usePathname();

  return (
    <motion.aside
      className={styles.sidebar}
      initial={{ opacity: 0, x: -6 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ duration: 0.18, delay: 0.04, ease: [0.16, 1, 0.3, 1] }}
    >
      <div className={styles.header}>
        <Link href="/" className={styles.logo}>
          <div className={styles.logoMark}>
            <span className={styles.logoSlash} />
            G
          </div>
          <span className={styles.logoText}>GUTS</span>
        </Link>
        <SyncStatus status="synced" />
      </div>

      <nav className={styles.nav}>
        <ul className={styles.navList}>
          {navItems.map((item) => {
            const isActive =
              item.href === "/"
                ? pathname === "/"
                : pathname.startsWith(item.href);

            return (
              <li key={item.href}>
                <Link
                  href={item.href}
                  className={clsx(styles.navItem, isActive && styles.active)}
                >
                  <span className={styles.navIcon}>{item.icon}</span>
                  <span className={styles.navLabel}>{item.label}</span>
                  {isActive && <span className={styles.slashNotch} />}
                </Link>
              </li>
            );
          })}
        </ul>
      </nav>

      <div className={styles.bottom}>
        <ul className={styles.navList}>
          {bottomItems.map((item) => {
            const isActive = pathname.startsWith(item.href);

            return (
              <li key={item.href}>
                <Link
                  href={item.href}
                  className={clsx(styles.navItem, isActive && styles.active)}
                >
                  <span className={styles.navIcon}>{item.icon}</span>
                  <span className={styles.navLabel}>{item.label}</span>
                  {isActive && <span className={styles.slashNotch} />}
                </Link>
              </li>
            );
          })}
        </ul>
        <a
          href="https://github.com/guts"
          target="_blank"
          rel="noopener noreferrer"
          className={styles.helpLink}
        >
          <svg viewBox="0 0 20 20" fill="none">
            <circle cx="10" cy="10" r="7" stroke="currentColor" strokeWidth="1.5" />
            <path
              d="M7.5 7.5a2.5 2.5 0 015 0c0 1.5-1.5 2-1.5 3M10 14h.01"
              stroke="currentColor"
              strokeWidth="1.5"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </svg>
          <span>Help</span>
        </a>
      </div>
    </motion.aside>
  );
}
