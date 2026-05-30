import type { ReactNode } from "react";

interface NeoCardProps {
  title?: string;
  children: ReactNode;
  className?: string;
}

export function NeoCard({ title, children, className = "" }: NeoCardProps) {
  return (
    <section className={`neo-surface p-6 ${className}`}>
      {title ? (
        <h2 className="mb-4 text-lg font-semibold text-[var(--neo-text)]">{title}</h2>
      ) : null}
      {children}
    </section>
  );
}
