"use client";

import * as AvatarPrimitive from "@radix-ui/react-avatar";
import clsx from "clsx";
import styles from "./Avatar.module.css";

interface AvatarProps {
  src?: string;
  alt?: string;
  fallback: string;
  size?: "sm" | "md" | "lg";
  className?: string;
}

export function Avatar({
  src,
  alt,
  fallback,
  size = "md",
  className,
}: AvatarProps) {
  return (
    <AvatarPrimitive.Root
      className={clsx(styles.root, styles[size], className)}
    >
      <AvatarPrimitive.Image src={src} alt={alt} className={styles.image} />
      <AvatarPrimitive.Fallback className={styles.fallback} delayMs={600}>
        {fallback}
      </AvatarPrimitive.Fallback>
    </AvatarPrimitive.Root>
  );
}

interface AvatarGroupProps {
  children: React.ReactNode;
  max?: number;
  className?: string;
}

export function AvatarGroup({ children, max, className }: AvatarGroupProps) {
  const childArray = Array.isArray(children) ? children : [children];
  const visibleChildren = max ? childArray.slice(0, max) : childArray;
  const remaining = max ? childArray.length - max : 0;

  return (
    <div className={clsx(styles.group, className)}>
      {visibleChildren}
      {remaining > 0 && (
        <span className={styles.remaining}>+{remaining}</span>
      )}
    </div>
  );
}
