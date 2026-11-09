import { cn } from "cn"
import { NavLink, Outlet } from "react-router"
import { Logo } from "./Logo"

const navItems = [
  { to: "/runs", label: "Runs" },
  { to: "/jobs", label: "Jobs" },
]

export function Layout() {
  return (
    <div className="mx-auto flex max-w-7xl flex-col gap-6 px-8 pt-4 pb-10">
      <header className="glass flex flex-wrap items-center gap-2 rounded-[28px] px-3 py-2">
        <div className="mr-4 flex items-center gap-2 pl-2">
          <Logo className="size-6 text-primary" />
          <span className="text-base font-semibold">Flare</span>
        </div>
        <nav className="flex gap-1">
          {navItems.map((item) => (
            <NavLink
              key={item.to}
              to={item.to}
              className={({ isActive }) =>
                cn(
                  "flex h-10 items-center rounded-[20px] px-4 text-sm transition-colors duration-150",
                  isActive
                    ? "bg-selected font-medium text-foreground"
                    : "text-muted-foreground hover:bg-accent",
                )
              }
            >
              {item.label}
            </NavLink>
          ))}
        </nav>
      </header>
      <main>
        <Outlet />
      </main>
    </div>
  )
}
