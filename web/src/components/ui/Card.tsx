"use client";

import { ReactNode, forwardRef, HTMLAttributes } from "react";
import { motion, HTMLMotionProps } from "framer-motion";
import clsx from "clsx";
import styles from "./Card.module.css";

interface CardProps extends HTMLAttributes<HTMLDivElement> {
  children: ReactNode;
  variant?: "default" | "interactive" | "elevated";
  padding?: "none" | "sm" | "md" | "lg";
  className?: string;
}

export const Card = forwardRef<HTMLDivElement, CardProps>(
  (
    { children, variant = "default", padding = "md", className, ...props },
    ref
  ) => {
    return (
      <div
        ref={ref}
        className={clsx(
          styles.card,
          styles[variant],
          styles[`padding-${padding}`],
          className
        )}
        {...props}
      >
        {children}
      </div>
    );
  }
);

Card.displayName = "Card";

interface InteractiveCardProps extends Omit<HTMLMotionProps<"div">, "padding"> {
  children: ReactNode;
  padding?: "none" | "sm" | "md" | "lg";
  selected?: boolean;
}

export const InteractiveCard = forwardRef<HTMLDivElement, InteractiveCardProps>(
  ({ children, padding = "md", selected = false, className, ...props }, ref) => {
    return (
      <motion.div
        ref={ref}
        className={clsx(
          styles.card,
          styles.interactive,
          styles[`padding-${padding}`],
          selected && styles.selected,
          className
        )}
        whileHover={{ y: -1, borderColor: "rgba(255,255,255,0.14)" }}
        transition={{ duration: 0.12, ease: [0.2, 0.9, 0.2, 1] }}
        {...props}
      >
        {children}
      </motion.div>
    );
  }
);

InteractiveCard.displayName = "InteractiveCard";

interface CardHeaderProps {
  children: ReactNode;
  actions?: ReactNode;
  className?: string;
}

export function CardHeader({ children, actions, className }: CardHeaderProps) {
  return (
    <div className={clsx(styles.header, className)}>
      <div className={styles.headerContent}>{children}</div>
      {actions && <div className={styles.headerActions}>{actions}</div>}
    </div>
  );
}

interface CardTitleProps {
  children: ReactNode;
  as?: "h1" | "h2" | "h3" | "h4" | "h5" | "h6";
  className?: string;
}

export function CardTitle({
  children,
  as: Tag = "h3",
  className,
}: CardTitleProps) {
  return <Tag className={clsx(styles.title, className)}>{children}</Tag>;
}

interface CardDescriptionProps {
  children: ReactNode;
  className?: string;
}

export function CardDescription({ children, className }: CardDescriptionProps) {
  return <p className={clsx(styles.description, className)}>{children}</p>;
}

interface CardContentProps {
  children: ReactNode;
  className?: string;
}

export function CardContent({ children, className }: CardContentProps) {
  return <div className={clsx(styles.content, className)}>{children}</div>;
}

interface CardFooterProps {
  children: ReactNode;
  className?: string;
}

export function CardFooter({ children, className }: CardFooterProps) {
  return <div className={clsx(styles.footer, className)}>{children}</div>;
}
