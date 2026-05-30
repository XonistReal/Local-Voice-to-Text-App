import { NavLink, Outlet } from "react-router-dom";
import { TitleBar } from "./TitleBar";

const links = [
  { to: "/", label: "Home" },
  { to: "/settings", label: "Settings" },
  { to: "/history", label: "History" },
];

export function Layout() {
  return (
    <div className="flex h-screen flex-col overflow-hidden bg-[var(--neo-bg)]">
      <TitleBar />
      <div className="neo-scroll flex-1 overflow-y-auto p-6">
        <div className="mx-auto flex max-w-3xl flex-col gap-6">
        <header className="flex flex-wrap items-center justify-between gap-4">
          <div>
            <p className="text-sm font-medium uppercase tracking-wider text-[var(--neo-muted)]">
              Local voice-to-text
            </p>
            <h1 className="text-3xl font-bold text-[var(--neo-text)]">VTT</h1>
          </div>
          <nav className="neo-surface flex gap-1 p-1">
            {links.map((link) => (
              <NavLink
                key={link.to}
                to={link.to}
                className={({ isActive }) =>
                  `rounded-xl px-4 py-2 text-sm font-medium transition-all ${
                    isActive
                      ? "neo-inset text-[var(--neo-text)]"
                      : "text-[var(--neo-muted)] hover:text-[var(--neo-text)]"
                  }`
                }
                end={link.to === "/"}
              >
                {link.label}
              </NavLink>
            ))}
          </nav>
        </header>
        <Outlet />
        </div>
      </div>
    </div>
  );
}
