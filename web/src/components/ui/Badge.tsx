"use client";

import { ReactNode } from "react";
import clsx from "clsx";
import styles from "./Badge.module.css";

type BadgeVariant =
  | "default"
  | "success"
  | "warning"
  | "danger"
  | "cipher"
  | "muted";

interface BadgeProps {
  children: ReactNode;
  variant?: BadgeVariant;
  size?: "sm" | "md";
  dot?: boolean;
  icon?: ReactNode;
  className?: string;
}

export function Badge({
  children,
  variant = "default",
  size = "md",
  dot = false,
  icon,
  className,
}: BadgeProps) {
  return (
    <span
      className={clsx(styles.badge, styles[variant], styles[size], className)}
    >
      {dot && <span className={styles.dot} />}
      {icon && <span className={styles.icon}>{icon}</span>}
      {children}
    </span>
  );
}

interface ConsensusSealProps {
  status: "verified" | "pending" | "conflicted";
  quorumSize?: number;
  nodeCount?: number;
  timestamp?: string;
  className?: string;
}

export function ConsensusSeal({
  status,
  quorumSize = 7,
  nodeCount = 12,
  timestamp,
  className,
}: ConsensusSealProps) {
  const statusConfig = {
    verified: {
      label: "Verified by quorum",
      variant: "cipher" as BadgeVariant,
      icon: (
        <svg viewBox="0 0 16 16" fill="none">
          <path
            d="M8 1L2 4v4c0 3.5 2.5 6.5 6 7.5 3.5-1 6-4 6-7.5V4L8 1z"
            stroke="currentColor"
            strokeWidth="1.5"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
          <path
            d="M6 8l1.5 1.5L10 6"
            stroke="currentColor"
            strokeWidth="1.5"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </svg>
      ),
    },
    pending: {
      label: "Pending consensus",
      variant: "warning" as BadgeVariant,
      icon: (
        <svg viewBox="0 0 16 16" fill="none">
          <circle
            cx="8"
            cy="8"
            r="6"
            stroke="currentColor"
            strokeWidth="1.5"
          />
          <path
            d="M8 5v3l2 1"
            stroke="currentColor"
            strokeWidth="1.5"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </svg>
      ),
    },
    conflicted: {
      label: "Conflict detected",
      variant: "danger" as BadgeVariant,
      icon: (
        <svg viewBox="0 0 16 16" fill="none">
          <path
            d="M8 1L1 14h14L8 1z"
            stroke="currentColor"
            strokeWidth="1.5"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
          <path
            d="M8 6v3M8 11.5v.5"
            stroke="currentColor"
            strokeWidth="1.5"
            strokeLinecap="round"
          />
        </svg>
      ),
    },
  };

  const config = statusConfig[status];

  return (
    <div className={clsx(styles.consensusSeal, className)}>
      <Badge variant={config.variant} icon={config.icon}>
        {config.label}
      </Badge>
      <div className={styles.sealTooltip}>
        <div className={styles.sealTooltipRow}>
          <span className={styles.sealTooltipLabel}>Quorum size</span>
          <span className={styles.sealTooltipValue}>{quorumSize} nodes</span>
        </div>
        <div className={styles.sealTooltipRow}>
          <span className={styles.sealTooltipLabel}>Network observers</span>
          <span className={styles.sealTooltipValue}>{nodeCount} nodes</span>
        </div>
        {timestamp && (
          <div className={styles.sealTooltipRow}>
            <span className={styles.sealTooltipLabel}>Last verified</span>
            <span className={styles.sealTooltipValue}>{timestamp}</span>
          </div>
        )}
      </div>
    </div>
  );
}

interface SyncStatusProps {
  status: "synced" | "syncing" | "degraded" | "offline";
  className?: string;
}

export function SyncStatus({ status, className }: SyncStatusProps) {
  const statusConfig = {
    synced: { label: "Synced", variant: "success" as BadgeVariant },
    syncing: { label: "Syncing...", variant: "warning" as BadgeVariant },
    degraded: { label: "Degraded", variant: "warning" as BadgeVariant },
    offline: { label: "Offline", variant: "danger" as BadgeVariant },
  };

  const config = statusConfig[status];

  return (
    <Badge variant={config.variant} size="sm" dot className={className}>
      {config.label}
    </Badge>
  );
}
