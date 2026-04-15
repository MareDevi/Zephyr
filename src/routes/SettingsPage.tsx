import { Card, Label, ListBox, Select, Slider, Switch, Tabs } from "@heroui/react";
import { useEffect, useState } from "react";
import {
	commands,
	type ColorMode,
	type LaunchBehavior,
	type SettingsSnapshotDto,
	type ThemeId,
	type ThemeKind,
} from "../bindings";
import { type ThemeConfig, useTheme } from "../features/theme/ThemeProvider";

const CATPPUCCIN_OPTIONS: ThemeId[] = ["latte", "frappe", "macchiato", "mocha"];
const THEME_KIND_OPTIONS: ThemeKind[] = ["heroui", "catppuccin"];
const COLOR_MODE_OPTIONS: ColorMode[] = ["system", "light", "dark"];
const LAUNCH_BEHAVIOR_OPTIONS: LaunchBehavior[] = ["normal", "silent"];
const DEFAULT_ACCENT_COLOR = "#89b4fa";
const DEFAULT_ACCENT_HUE = 217;

function normalizeSettings(raw?: SettingsSnapshotDto | null): SettingsSnapshotDto {
	return {
		autostartEnabled: raw?.autostartEnabled ?? false,
		launchBehavior: raw?.launchBehavior ?? "normal",
		themeKind: raw?.themeKind ?? "heroui",
		themeId: raw?.themeId ?? "default",
		accentColor: raw?.accentColor ?? null,
		colorMode: raw?.colorMode ?? "system",
	};
}

function formatThemeLabel(themeId: ThemeId): string {
	return themeId.charAt(0).toUpperCase() + themeId.slice(1);
}

function formatThemeKindLabel(themeKind: ThemeKind): string {
	return themeKind === "heroui" ? "HeroUI" : "Catppuccin";
}

function formatColorModeLabel(value: ColorMode): string {
	if (value === "system") return "Follow system";
	return value === "light" ? "Light" : "Dark";
}

function formatLaunchBehaviorLabel(value: LaunchBehavior): string {
	return value === "silent" ? "Silent launch (start in tray)" : "Normal launch";
}

export function SettingsPage() {
	const { themeKind, themeId, accentColor, colorMode, setThemeConfig } =
		useTheme();
	const [settings, setSettings] = useState<SettingsSnapshotDto>(normalizeSettings());
	const [isBusy, setIsBusy] = useState(false);
	const [errorMessage, setErrorMessage] = useState<string | null>(null);
	const [customAccentEnabled, setCustomAccentEnabled] = useState(false);
	const [customAccentValue, setCustomAccentValue] = useState(DEFAULT_ACCENT_COLOR);
	const [accentHue, setAccentHue] = useState(DEFAULT_ACCENT_HUE);

	useEffect(() => {
		let cancelled = false;
		const loadSettings = async () => {
			try {
				const result = await commands.getSettings();
				if (cancelled) return;
				if (result.status === "error") {
					setErrorMessage(result.error.message);
					return;
				}
				const next = normalizeSettings(result.data);
				setSettings(next);
				setCustomAccentEnabled(next.themeKind === "heroui" && !!next.accentColor);
				setCustomAccentValue(next.accentColor ?? DEFAULT_ACCENT_COLOR);
				setAccentHue(hexToHue(next.accentColor ?? DEFAULT_ACCENT_COLOR));
			} catch (error) {
				if (!cancelled) {
					setErrorMessage(error instanceof Error ? error.message : String(error));
				}
			}
		};
		void loadSettings();
		return () => {
			cancelled = true;
		};
	}, []);

	const runUpdate = async (operation: () => Promise<void>) => {
		setIsBusy(true);
		setErrorMessage(null);
		try {
			await operation();
		} catch (error) {
			setErrorMessage(error instanceof Error ? error.message : String(error));
		} finally {
			setIsBusy(false);
		}
	};

	const applyTheme = async (nextConfig: ThemeConfig) => {
		await setThemeConfig(nextConfig);
		setSettings((prev) =>
			normalizeSettings({
				...prev,
				themeKind: nextConfig.themeKind,
				themeId: nextConfig.themeId,
				accentColor: nextConfig.accentColor,
				colorMode: nextConfig.colorMode,
			}),
		);
	};

	return (
		<div className="space-y-6">
			<Card className="dashboard-card p-6">
				<div className="space-y-5">
					<div>
						<h3 className="text-lg font-bold">Startup</h3>
						<p className="text-xs text-default-500 mt-1">
							Configure start-on-boot and application startup behavior.
						</p>
					</div>

					<Switch
						isSelected={settings.autostartEnabled}
						onChange={(enabled) =>
							void runUpdate(async () => {
								const result = await commands.setAutostart({ enabled });
								if (result.status === "error") {
									setErrorMessage(result.error.message);
									return;
								}
								setSettings(normalizeSettings(result.data));
							})
						}
						isDisabled={isBusy}
					>
						<Switch.Control>
							<Switch.Thumb />
						</Switch.Control>
						<Switch.Content>
							<Label>Start on boot</Label>
						</Switch.Content>
					</Switch>

					<Select
						aria-label="Select launch behavior"
						selectedKey={settings.launchBehavior}
						onSelectionChange={(key) =>
							void runUpdate(async () => {
								const launchBehavior = String(key) as LaunchBehavior;
								const result = await commands.setLaunchBehavior({
									launchBehavior,
								});
								if (result.status === "error") {
									setErrorMessage(result.error.message);
									return;
								}
								setSettings(normalizeSettings(result.data));
							})
						}
						isDisabled={isBusy}
					>
						<Select.Trigger>
							<Select.Value />
							<Select.Indicator />
						</Select.Trigger>
						<Select.Popover>
							<ListBox
								aria-label="Launch behavior options"
								selectedKeys={[settings.launchBehavior ?? "normal"]}
							>
								{LAUNCH_BEHAVIOR_OPTIONS.map((value) => (
									<ListBox.Item key={value} id={value} textValue={value}>
										{formatLaunchBehaviorLabel(value)}
										<ListBox.ItemIndicator />
									</ListBox.Item>
								))}
							</ListBox>
						</Select.Popover>
					</Select>
				</div>
			</Card>

			<Card className="dashboard-card p-6">
				<div className="space-y-5">
					<div>
						<h3 className="text-lg font-bold">Theme</h3>
						<p className="text-xs text-default-500 mt-1">
							Use HeroUI default theming with optional custom accent, or switch to Catppuccin palettes.
						</p>
					</div>

					<Tabs
						className="w-full"
						selectedKey={themeKind}
						onSelectionChange={(key) =>
							void runUpdate(async () => {
								const nextKind = String(key) as ThemeKind;
								await applyTheme({
									themeKind: nextKind,
									themeId: nextKind === "catppuccin" ? "mocha" : "default",
									accentColor:
										nextKind === "heroui" && customAccentEnabled
											? customAccentValue
											: null,
									colorMode,
								});
							})
						}
					>
						<Tabs.ListContainer className="w-full">
							<Tabs.List
								aria-label="Theme system options"
								className="w-full *:data-[selected=true]:text-accent-foreground"
							>
								{THEME_KIND_OPTIONS.map((value) => (
									<Tabs.Tab key={value} id={value}>
										{formatThemeKindLabel(value)}
										<Tabs.Indicator className="bg-accent" />
									</Tabs.Tab>
								))}
							</Tabs.List>
						</Tabs.ListContainer>

						<Tabs.Panel className="pt-4" id="heroui">
							<div className="space-y-4">
								<Switch
									isSelected={customAccentEnabled}
									onChange={(enabled) =>
										void runUpdate(async () => {
											setCustomAccentEnabled(enabled);
											const nextAccent = enabled ? customAccentValue : null;
											await applyTheme({
												themeKind: "heroui",
												themeId: "default",
												accentColor: nextAccent,
												colorMode,
											});
										})
									}
									isDisabled={isBusy}
								>
									<Switch.Control>
										<Switch.Thumb />
									</Switch.Control>
									<Switch.Content>
										<Label>Use custom HeroUI accent</Label>
									</Switch.Content>
								</Switch>

								<Select
									aria-label="Select HeroUI color mode"
									selectedKey={colorMode}
									onSelectionChange={(key) =>
										void runUpdate(async () => {
											const nextColorMode = String(key) as ColorMode;
											await applyTheme({
												themeKind: "heroui",
												themeId: "default",
												accentColor: customAccentEnabled
													? customAccentValue
													: null,
												colorMode: nextColorMode,
											});
										})
									}
									isDisabled={isBusy}
								>
									<Select.Trigger>
										<Select.Value />
										<Select.Indicator />
									</Select.Trigger>
									<Select.Popover>
										<ListBox
											aria-label="HeroUI color mode options"
											selectedKeys={[colorMode]}
										>
											{COLOR_MODE_OPTIONS.map((value) => (
												<ListBox.Item
													key={value}
													id={value}
													textValue={value}
												>
													{formatColorModeLabel(value)}
													<ListBox.ItemIndicator />
												</ListBox.Item>
											))}
										</ListBox>
									</Select.Popover>
								</Select>

								{customAccentEnabled ? (
									<div className="space-y-3">
										<Slider
											className="w-full max-w-xs"
											minValue={0}
											maxValue={360}
											step={1}
											value={accentHue}
											onChange={(value) => {
												const nextHue = value as number;
												setAccentHue(nextHue);
												setCustomAccentValue(hueToHex(nextHue));
											}}
											isDisabled={isBusy}
										>
											<Label>Accent Hue</Label>
											<Slider.Output />
											<Slider.Track>
												<Slider.Fill />
												<Slider.Thumb />
											</Slider.Track>
										</Slider>

										<button
											type="button"
											className="rounded-lg border border-default-200 px-3 py-2 text-xs font-semibold text-default-600 hover:bg-default-100"
											onClick={() =>
												void runUpdate(async () => {
													await applyTheme({
														themeKind: "heroui",
														themeId: "default",
														accentColor: customAccentValue,
														colorMode,
													});
												})
											}
											disabled={isBusy}
										>
											Apply Accent {accentColor ? `(${accentColor})` : ""}
										</button>
									</div>
								) : null}
							</div>
						</Tabs.Panel>

						<Tabs.Panel className="pt-4" id="catppuccin">
							<Select
								aria-label="Select catppuccin theme"
								selectedKey={themeId}
								onSelectionChange={(key) =>
									void runUpdate(async () => {
										await applyTheme({
											themeKind: "catppuccin",
											themeId: String(key) as ThemeId,
											accentColor: null,
											colorMode,
										});
									})
								}
								isDisabled={isBusy}
							>
								<Select.Trigger>
									<Select.Value />
									<Select.Indicator />
								</Select.Trigger>
								<Select.Popover>
									<ListBox aria-label="Catppuccin theme options" selectedKeys={[themeId]}>
										{CATPPUCCIN_OPTIONS.map((value) => (
											<ListBox.Item key={value} id={value} textValue={value}>
												{formatThemeLabel(value)}
												<ListBox.ItemIndicator />
											</ListBox.Item>
										))}
									</ListBox>
								</Select.Popover>
							</Select>
						</Tabs.Panel>
					</Tabs>
				</div>
			</Card>

			{errorMessage ? (
				<p className="rounded-xl border border-danger/40 bg-danger-soft px-4 py-3 text-sm text-danger">
					{errorMessage}
				</p>
			) : null}
		</div>
	);
}

function hueToHex(hue: number): string {
	const normalized = ((hue % 360) + 360) % 360;
	const h = normalized / 60;
	const c = 1;
	const x = c * (1 - Math.abs((h % 2) - 1));
	let r = 0;
	let g = 0;
	let b = 0;
	if (h >= 0 && h < 1) [r, g, b] = [c, x, 0];
	else if (h < 2) [r, g, b] = [x, c, 0];
	else if (h < 3) [r, g, b] = [0, c, x];
	else if (h < 4) [r, g, b] = [0, x, c];
	else if (h < 5) [r, g, b] = [x, 0, c];
	else [r, g, b] = [c, 0, x];
	const toHex = (value: number) =>
		Math.round((value * 0.6 + 0.2) * 255)
			.toString(16)
			.padStart(2, "0");
	return `#${toHex(r)}${toHex(g)}${toHex(b)}`;
}

function hexToHue(value: string): number {
	if (!/^#[0-9a-fA-F]{6}$/.test(value)) return DEFAULT_ACCENT_HUE;
	const r = Number.parseInt(value.slice(1, 3), 16) / 255;
	const g = Number.parseInt(value.slice(3, 5), 16) / 255;
	const b = Number.parseInt(value.slice(5, 7), 16) / 255;
	const max = Math.max(r, g, b);
	const min = Math.min(r, g, b);
	const delta = max - min;
	if (delta === 0) return 0;
	let hue = 0;
	if (max === r) hue = ((g - b) / delta) % 6;
	else if (max === g) hue = (b - r) / delta + 2;
	else hue = (r - g) / delta + 4;
	const normalized = Math.round(hue * 60);
	return normalized < 0 ? normalized + 360 : normalized;
}
