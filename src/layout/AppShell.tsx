import { Button } from "@heroui/react";
import {
	IconAdjustments,
	IconDashboard,
	IconMoon,
	IconSun,
	IconWind,
} from "@tabler/icons-react";
import { Link, useLocation } from "@tanstack/react-router";
import type { ReactNode } from "react";
import { useTheme } from "../features/theme/ThemeProvider";

interface NavItem {
	to: string;
	label: string;
	icon: typeof IconDashboard;
}

const navItems: NavItem[] = [
	{ to: "/", label: "Dashboard", icon: IconDashboard },
	{ to: "/profiles", label: "Profiles", icon: IconWind },
	{ to: "/fancurves", label: "Fan Curves", icon: IconWind },
	{ to: "/aura", label: "Aura", icon: IconSun },
	{ to: "/advanced", label: "Advanced", icon: IconAdjustments },
];

export function AppShell({ children }: { children: ReactNode }) {
	const { mode, setMode } = useTheme();
	const location = useLocation();

	return (
		<div className="flex h-screen w-full overflow-hidden bg-background text-foreground">
			<aside className="flex w-64 flex-col border-r border-default-100 bg-background">
				<div className="p-6">
					<span className="text-lg font-bold">Zephyr</span>
				</div>

				<nav className="flex-1 space-y-1 px-3">
					{navItems.map((item) => {
						const active =
							item.to === "/"
								? location.pathname === "/"
								: location.pathname === item.to ||
									location.pathname.startsWith(`${item.to}/`);
						return (
							<Link
								key={item.to}
								to={item.to}
								className={`group flex items-center gap-3 rounded-xl px-3 py-2.5 text-sm transition-colors ${
									active
										? "bg-primary text-primary-foreground"
										: "text-default-500 hover:bg-default-100/50 hover:text-foreground"
								}`}
							>
								<item.icon
									size={20}
									className={active ? "text-primary-foreground" : "text-default-400"}
								/>
								<span className="flex-1 font-medium">{item.label}</span>
							</Link>
						);
					})}
				</nav>

				<div className="p-3">
					<Button
						isIconOnly
						variant="tertiary"
						aria-label={
							mode === "dark" ? "Switch to light mode" : "Switch to dark mode"
						}
						onPress={() => setMode(mode === "dark" ? "light" : "dark")}
					>
						{mode === "light" ? (
							<IconMoon size={20} className="text-default-500" />
						) : (
							<IconSun size={20} className="text-default-500" />
						)}
					</Button>
				</div>
			</aside>

			<main className="flex min-w-0 flex-1 flex-col overflow-hidden bg-background">
				<header className="flex h-16 items-center justify-between border-b border-default-100 px-8">
					<div>
						<h2 className="text-xl font-bold tracking-tight text-foreground">
							Zephyr
						</h2>
					</div>
				</header>

				<div className="min-h-0 flex-1 overflow-y-auto px-8 py-8">
					<div className="mx-auto max-w-7xl">{children}</div>
				</div>
			</main>
		</div>
	);
}
