"use client";

import * as TabsPrimitive from "@radix-ui/react-tabs";
import { motion } from "framer-motion";
import clsx from "clsx";
import styles from "./Tabs.module.css";

interface TabsProps {
  defaultValue: string;
  children: React.ReactNode;
  className?: string;
}

export function Tabs({ defaultValue, children, className }: TabsProps) {
  return (
    <TabsPrimitive.Root
      defaultValue={defaultValue}
      className={clsx(styles.root, className)}
    >
      {children}
    </TabsPrimitive.Root>
  );
}

interface TabsListProps {
  children: React.ReactNode;
  className?: string;
}

export function TabsList({ children, className }: TabsListProps) {
  return (
    <TabsPrimitive.List className={clsx(styles.list, className)}>
      {children}
    </TabsPrimitive.List>
  );
}

interface TabsTriggerProps {
  value: string;
  children: React.ReactNode;
  icon?: React.ReactNode;
  count?: number;
  className?: string;
}

export function TabsTrigger({
  value,
  children,
  icon,
  count,
  className,
}: TabsTriggerProps) {
  return (
    <TabsPrimitive.Trigger
      value={value}
      className={clsx(styles.trigger, className)}
    >
      {icon && <span className={styles.icon}>{icon}</span>}
      <span>{children}</span>
      {count !== undefined && <span className={styles.count}>{count}</span>}
      <span className={styles.slashNotch} aria-hidden="true" />
    </TabsPrimitive.Trigger>
  );
}

interface TabsContentProps {
  value: string;
  children: React.ReactNode;
  className?: string;
}

export function TabsContent({ value, children, className }: TabsContentProps) {
  return (
    <TabsPrimitive.Content
      value={value}
      className={clsx(styles.content, className)}
      asChild
    >
      <motion.div
        initial={{ opacity: 0, y: 4 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.18, ease: [0.16, 1, 0.3, 1] }}
      >
        {children}
      </motion.div>
    </TabsPrimitive.Content>
  );
}
