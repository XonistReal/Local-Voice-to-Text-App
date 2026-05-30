import type { ButtonHTMLAttributes, ReactNode } from "react";

interface NeoButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "default" | "accent" | "ghost";
  children: ReactNode;
}

export function NeoButton({
  variant = "default",
  className = "",
  children,
  ...props
}: NeoButtonProps) {
  const base =
    "inline-flex items-center justify-center gap-2 rounded-2xl px-5 py-3 font-medium transition-all disabled:opacity-50 disabled:cursor-not-allowed min-h-11";
  const styles = {
    default: "neo-surface hover:brightness-[1.02] active:neo-pressed text-[var(--neo-text)]",
    accent:
      "neo-surface text-white bg-[var(--neo-accent)] hover:bg-[var(--neo-accent-hover)] active:neo-pressed",
    ghost:
      "bg-transparent shadow-none hover:neo-surface active:neo-pressed text-[var(--neo-muted)]",
  };

  return (
    <button className={`${base} ${styles[variant]} ${className}`} {...props}>
      {children}
    </button>
  );
}
