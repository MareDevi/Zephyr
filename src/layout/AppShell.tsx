import { Button, Card } from "@heroui/react";
import {
	IconAdjustments,
	IconDashboard,
	IconLayoutSidebarLeftCollapse,
	IconMoon,
	IconSettings,
	IconSettingsAutomation,
	IconSun,
	IconWind,
} from "@tabler/icons-react";
import { Link, useLocation } from "@tanstack/react-router";
import { type ReactNode, useMemo, useState } from "react";
import { useTheme } from "../features/theme/ThemeProvider";

interface NavItem {
	to: string;
	label: string;
	description: string;
	icon: typeof IconDashboard;
}

const navItems: NavItem[] = [
	{
		to: "/",
		label: "Dashboard",
		description: "Live telemetry and quick status",
		icon: IconDashboard,
	},
	{
		to: "/profiles",
		label: "Profiles",
		description: "Platform, GPU, and charge policy",
		icon: IconSettingsAutomation,
	},
	{
		to: "/fancurves",
		label: "Fan Curves",
		description: "Tune thermal response curve",
		icon: IconWind,
	},
	{
		to: "/aura",
		label: "Lighting",
		description: "Aura and Slash configuration",
		icon: IconSun,
	},
	{
		to: "/advanced",
		label: "Advanced",
		description: "Backlight, SCSI, and Armoury",
		icon: IconAdjustments,
	},
	{
		to: "/settings",
		label: "Settings",
		description: "Startup behavior and themes",
		icon: IconSettings,
	},
];

export function AppShell({ children }: { children: ReactNode }) {
	const location = useLocation();
	const [collapsed, setCollapsed] = useState(false);
	const { themeKind, themeId, accentColor, effectiveMode, setThemeConfig } =
		useTheme();

	const activeItem = useMemo(
		() =>
			navItems.find((item) =>
				item.to === "/"
					? location.pathname === "/"
					: location.pathname === item.to ||
						location.pathname.startsWith(`${item.to}/`),
			) ?? navItems[0],
		[location.pathname],
	);

	return (
		<div className="flex h-screen w-full overflow-hidden bg-background text-foreground">
			<aside
				className={`flex flex-col border-r border-default-200 bg-background backdrop-blur-sm transition-all duration-300 motion-reduce:transition-none ${
					collapsed ? "w-[4.5rem]" : "w-[var(--app-sidebar-width)]"
				}`}
			>
				<div className="flex items-center justify-between p-4">
					{collapsed ? null : (
						<div className="flex flex-col">
							<span className="text-sm font-semibold text-default-500">
								Control Hub
							</span>
							<span className="text-lg font-bold tracking-tight">Zephyr</span>
						</div>
					)}
					<Button
						isIconOnly
						size="sm"
						aria-label={collapsed ? "Expand sidebar" : "Collapse sidebar"}
						onPress={() => setCollapsed((prev) => !prev)}
					>
						<IconLayoutSidebarLeftCollapse size={16} />
					</Button>
				</div>

				<nav className="flex-1 space-y-1 px-2 pb-4">
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
								className={`group flex items-center gap-3 rounded-xl px-3 py-2.5 text-sm transition-all duration-200 motion-reduce:transition-none ${
									active
										? "bg-accent text-accent-foreground shadow-sm"
										: "text-default-500 hover:bg-default-100/60 hover:text-foreground"
								}`}
							>
								<item.icon
									size={18}
									className={
										active ? "text-accent-foreground" : "text-default-400"
									}
								/>
								{collapsed ? null : (
									<span className="flex-1">
										<span className="block font-medium leading-tight">
											{item.label}
										</span>
										<span
											className={`mt-0.5 block text-[10px] leading-tight ${
												active ? "text-accent-foreground" : "text-default-400"
											}`}
										>
											{item.description}
										</span>
									</span>
								)}
							</Link>
						);
					})}
				</nav>

				<div className="space-y-3 p-3">
					{collapsed ? null : (
						<Card className="dashboard-card p-3">
							<div className="flex items-center justify-between gap-2">
								<div className="text-xs">
									<p className="font-semibold">Current view</p>
									<p className="text-default-500">{activeItem.label}</p>
								</div>
								<span className="status-pill border-success bg-default text-success">
									Ready
								</span>
							</div>
						</Card>
					)}
					{themeKind === "heroui" ? (
						<Button
							isIconOnly
							className={collapsed ? "mx-auto" : undefined}
							aria-label={
								effectiveMode === "dark"
									? "Switch to light mode"
									: "Switch to dark mode"
							}
							onPress={() =>
								void setThemeConfig({
									themeKind,
									themeId,
									accentColor,
									colorMode: effectiveMode === "dark" ? "light" : "dark",
								})
							}
						>
							{effectiveMode === "dark" ? (
								<IconSun size={20} />
							) : (
								<IconMoon size={20} />
							)}
						</Button>
					) : null}
				</div>
			</aside>

			<main className="app-canvas flex min-w-0 flex-1 flex-col overflow-hidden">
				<header className="flex h-16 items-center justify-between border-b border-default-200 bg-content1 px-6 backdrop-blur-sm">
					<div>
						<h2 className="text-lg font-semibold tracking-tight">
							{activeItem.label}
						</h2>
						<p className="text-xs text-default-500">{activeItem.description}</p>
					</div>
				</header>

				<div className="min-h-0 flex-1 overflow-y-auto px-6 py-6">
					<div className="mx-auto w-full max-w-[var(--app-shell-max-width)]">
						{children}
					</div>
				</div>
			</main>
		</div>
	);
}
