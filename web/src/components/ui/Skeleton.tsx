"use client";

import clsx from "clsx";
import styles from "./Skeleton.module.css";

interface SkeletonProps {
  width?: string | number;
  height?: string | number;
  rounded?: "none" | "sm" | "md" | "lg" | "full";
  className?: string;
}

export function Skeleton({
  width,
  height,
  rounded = "md",
  className,
}: SkeletonProps) {
  return (
    <div
      className={clsx(styles.skeleton, styles[`rounded-${rounded}`], className)}
      style={{
        width: typeof width === "number" ? `${width}px` : width,
        height: typeof height === "number" ? `${height}px` : height,
      }}
    />
  );
}

export function SkeletonText({ lines = 3 }: { lines?: number }) {
  return (
    <div className={styles.textWrapper}>
      {Array.from({ length: lines }).map((_, i) => (
        <Skeleton
          key={i}
          height={14}
          width={i === lines - 1 ? "60%" : "100%"}
          rounded="sm"
        />
      ))}
    </div>
  );
}

export function SkeletonCard() {
  return (
    <div className={styles.card}>
      <div className={styles.cardHeader}>
        <Skeleton width={40} height={40} rounded="full" />
        <div className={styles.cardHeaderText}>
          <Skeleton width={120} height={14} rounded="sm" />
          <Skeleton width={80} height={12} rounded="sm" />
        </div>
      </div>
      <SkeletonText lines={2} />
      <div className={styles.cardFooter}>
        <Skeleton width={60} height={24} rounded="md" />
        <Skeleton width={60} height={24} rounded="md" />
      </div>
    </div>
  );
}

export function SkeletonTable({ rows = 5 }: { rows?: number }) {
  return (
    <div className={styles.table}>
      <div className={styles.tableHeader}>
        <Skeleton width="15%" height={12} rounded="sm" />
        <Skeleton width="30%" height={12} rounded="sm" />
        <Skeleton width="20%" height={12} rounded="sm" />
        <Skeleton width="15%" height={12} rounded="sm" />
      </div>
      {Array.from({ length: rows }).map((_, i) => (
        <div key={i} className={styles.tableRow}>
          <Skeleton width="15%" height={14} rounded="sm" />
          <Skeleton width="30%" height={14} rounded="sm" />
          <Skeleton width="20%" height={14} rounded="sm" />
          <Skeleton width="15%" height={14} rounded="sm" />
        </div>
      ))}
    </div>
  );
}
