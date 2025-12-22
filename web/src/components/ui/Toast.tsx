"use client";

import * as ToastPrimitive from "@radix-ui/react-toast";
import { motion, AnimatePresence } from "framer-motion";
import clsx from "clsx";
import styles from "./Toast.module.css";

export const ToastProvider = ToastPrimitive.Provider;
export const ToastViewport = () => (
  <ToastPrimitive.Viewport className={styles.viewport} />
);

type ToastVariant = "default" | "success" | "warning" | "error";

interface ToastProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  description?: string;
  variant?: ToastVariant;
  action?: React.ReactNode;
}

export function Toast({
  open,
  onOpenChange,
  title,
  description,
  variant = "default",
  action,
}: ToastProps) {
  const icons = {
    default: (
      <svg viewBox="0 0 20 20" fill="none">
        <circle cx="10" cy="10" r="8" stroke="currentColor" strokeWidth="1.5" />
        <path
          d="M10 7v3M10 13h.01"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
        />
      </svg>
    ),
    success: (
      <svg viewBox="0 0 20 20" fill="none">
        <circle cx="10" cy="10" r="8" stroke="currentColor" strokeWidth="1.5" />
        <path
          d="M7 10l2 2 4-4"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      </svg>
    ),
    warning: (
      <svg viewBox="0 0 20 20" fill="none">
        <path
          d="M10 3L2 17h16L10 3z"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
        <path
          d="M10 8v3M10 14h.01"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
        />
      </svg>
    ),
    error: (
      <svg viewBox="0 0 20 20" fill="none">
        <circle cx="10" cy="10" r="8" stroke="currentColor" strokeWidth="1.5" />
        <path
          d="M12.5 7.5l-5 5M7.5 7.5l5 5"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
        />
      </svg>
    ),
  };

  return (
    <AnimatePresence>
      {open && (
        <ToastPrimitive.Root
          open={open}
          onOpenChange={onOpenChange}
          asChild
          duration={4000}
        >
          <motion.div
            className={clsx(styles.toast, styles[variant])}
            initial={{ opacity: 0, y: 20, scale: 0.95 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: 10, scale: 0.95 }}
            transition={{ duration: 0.18, ease: [0.16, 1, 0.3, 1] }}
          >
            <div className={styles.icon}>{icons[variant]}</div>
            <div className={styles.content}>
              <ToastPrimitive.Title className={styles.title}>
                {title}
              </ToastPrimitive.Title>
              {description && (
                <ToastPrimitive.Description className={styles.description}>
                  {description}
                </ToastPrimitive.Description>
              )}
            </div>
            {action && (
              <ToastPrimitive.Action altText="action" asChild>
                {action}
              </ToastPrimitive.Action>
            )}
            <ToastPrimitive.Close className={styles.close} aria-label="Close">
              <svg viewBox="0 0 16 16" fill="none">
                <path
                  d="M12 4L4 12M4 4l8 8"
                  stroke="currentColor"
                  strokeWidth="1.5"
                  strokeLinecap="round"
                />
              </svg>
            </ToastPrimitive.Close>
          </motion.div>
        </ToastPrimitive.Root>
      )}
    </AnimatePresence>
  );
}
