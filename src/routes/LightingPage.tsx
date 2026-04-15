import {
	Button,
	Card,
	Input,
	Label,
	ListBox,
	Select,
	Separator,
	Slider,
	Switch,
} from "@heroui/react";
import { IconBolt, IconPalette } from "@tabler/icons-react";
import { useEffect, useMemo, useState } from "react";
import { commands } from "../bindings";
import {
	extractOptions,
	useDashboardRuntime,
} from "../features/dashboard/runtime";

const AURA_MODE_FALLBACK = [
	"Static",
	"Breathe",
	"RainbowCycle",
	"RainbowWave",
	"Pulse",
];
const SLASH_MODE_FALLBACK = [
	"Static",
	"Bounce",
	"Slash",
	"Loading",
	"BitStream",
	"Flow",
	"Spectrum",
];

function clampByte(value: string): number {
	return Math.max(0, Math.min(255, Number.parseInt(value, 10) || 0));
}

export function LightingPage() {
	const { snapshot, busyAction, runDashboardAction } = useDashboardRuntime();

	const [auraBrightness, setAuraBrightness] = useState("2");
	const [auraMode, setAuraMode] = useState("Static");
	const [slashEnabled, setSlashEnabled] = useState(true);
	const [slashBrightness, setSlashBrightness] = useState("128");
	const [slashInterval, setSlashInterval] = useState("0");
	const [slashMode, setSlashMode] = useState("Static");
	const [slashShowOnBoot, setSlashShowOnBoot] = useState(false);
	const [slashShowOnSleep, setSlashShowOnSleep] = useState(false);
	const [slashShowOnShutdown, setSlashShowOnShutdown] = useState(false);
	const [slashShowOnBattery, setSlashShowOnBattery] = useState(false);
	const [slashShowBatteryWarning, setSlashShowBatteryWarning] = useState(false);
	const [slashShowOnLidClosed, setSlashShowOnLidClosed] = useState(false);

	useEffect(() => {
		if (!snapshot) {
			return;
		}
		if (snapshot.aura.brightness) {
			const match = snapshot.aura.brightness.match(/\d+/);
			if (match) {
				setAuraBrightness(match[0]);
			}
		}
		if (snapshot.aura.ledMode) {
			setAuraMode(snapshot.aura.ledMode.replace(/"/g, ""));
		}
		if (snapshot.slash.enabled != null) {
			setSlashEnabled(snapshot.slash.enabled);
		}
		if (snapshot.slash.brightness != null) {
			setSlashBrightness(String(snapshot.slash.brightness));
		}
		if (snapshot.slash.interval != null) {
			setSlashInterval(String(snapshot.slash.interval));
		}
		if (snapshot.slash.mode) {
			setSlashMode(snapshot.slash.mode.replace(/"/g, ""));
		}
		if (snapshot.slash.showOnBoot != null) {
			setSlashShowOnBoot(snapshot.slash.showOnBoot);
		}
		if (snapshot.slash.showOnSleep != null) {
			setSlashShowOnSleep(snapshot.slash.showOnSleep);
		}
		if (snapshot.slash.showOnShutdown != null) {
			setSlashShowOnShutdown(snapshot.slash.showOnShutdown);
		}
		if (snapshot.slash.showOnBattery != null) {
			setSlashShowOnBattery(snapshot.slash.showOnBattery);
		}
		if (snapshot.slash.showBatteryWarning != null) {
			setSlashShowBatteryWarning(snapshot.slash.showBatteryWarning);
		}
		if (snapshot.slash.showOnLidClosed != null) {
			setSlashShowOnLidClosed(snapshot.slash.showOnLidClosed);
		}
	}, [snapshot]);

	const auraModes = useMemo(
		() =>
			extractOptions(snapshot?.aura.supportedBasicModes, AURA_MODE_FALLBACK),
		[snapshot?.aura.supportedBasicModes],
	);
	const slashModes = useMemo(() => {
		const selected = snapshot?.slash.mode?.replace(/"/g, "");
		if (selected && !SLASH_MODE_FALLBACK.includes(selected)) {
			return [selected, ...SLASH_MODE_FALLBACK];
		}
		return SLASH_MODE_FALLBACK;
	}, [snapshot?.slash.mode]);

	const auraControlEnabled = snapshot?.interfaces.asusdAuraAvailable ?? false;
	const slashControlEnabled = snapshot?.interfaces.asusdSlashAvailable ?? false;

	return (
		<div className="space-y-8">
			<div className="grid gap-6 md:grid-cols-2">
				{/* Keyboard Aura Card */}
				<Card className="dashboard-card p-6">
					<div className="flex flex-col gap-6">
						<div className="flex items-center justify-between">
							<div className="flex items-center gap-3">
								<div className="rounded-lg bg-default p-2 text-primary">
									<IconPalette size={20} />
								</div>
								<div>
									<h3 className="text-lg font-bold">Keyboard Aura</h3>
									<p className="text-xs text-default-500">
										Configure RGB effects and brightness.
									</p>
								</div>
							</div>
							<div
								className={`rounded-full px-2 py-0.5 text-[10px] font-bold uppercase ${
									auraControlEnabled
										? "bg-default text-success"
										: "bg-default text-danger"
								}`}
							>
								{auraControlEnabled ? "Available" : "Unavailable"}
							</div>
						</div>

						<Separator />

						<div className="space-y-8 py-2">
							{!auraControlEnabled ? (
								<p className="text-sm text-default-500">
									Aura controls are unavailable on this device or backend.
								</p>
							) : null}
							<Slider
								minValue={0}
								maxValue={3}
								step={1}
								value={Number.parseInt(auraBrightness, 10) || 0}
								onChange={(val) => setAuraBrightness(String(val))}
								isDisabled={!auraControlEnabled || !!busyAction}
							>
								<Label>Brightness</Label>
								<Slider.Output className="text-sm font-bold" />
								<Slider.Track>
									<Slider.Fill />
									<Slider.Thumb />
								</Slider.Track>
							</Slider>

							<Select
								selectedKey={auraMode}
								onSelectionChange={(key) => setAuraMode(String(key))}
								isDisabled={!auraControlEnabled || !!busyAction}
							>
								<Label>Effect Mode</Label>
								<Select.Trigger>
									<Select.Value />
									<Select.Indicator />
								</Select.Trigger>
								<Select.Popover>
									<ListBox
										aria-label="Aura mode options"
										selectedKeys={[auraMode]}
										onSelectionChange={(keys) => {
											const key = Array.from(keys)[0];
											if (key) setAuraMode(String(key));
										}}
									>
										{auraModes.map((mode) => (
											<ListBox.Item key={mode} id={mode} textValue={mode}>
												{mode}
												<ListBox.ItemIndicator />
											</ListBox.Item>
										))}
									</ListBox>
								</Select.Popover>
							</Select>
						</div>

						<div className="grid grid-cols-2 gap-3 mt-auto">
							<Button
								className="font-bold"
								isDisabled={!auraControlEnabled || !!busyAction}
								onPress={() =>
									runDashboardAction("setAuraBrightness", () =>
										commands.setAuraBrightness({
											level: Number.parseInt(auraBrightness, 10) || 2,
										}),
									)
								}
							>
								Apply Brightness
							</Button>
							<Button
								className="font-bold"
								isDisabled={!auraControlEnabled || !!busyAction}
								onPress={() =>
									runDashboardAction("setAuraMode", () =>
										commands.setAuraMode({ mode: auraMode }),
									)
								}
							>
								Apply Mode
							</Button>
						</div>
					</div>
				</Card>

				{/* Slash Lighting Card */}
				<Card className="dashboard-card p-6">
					<div className="flex flex-col gap-6">
						<div className="flex items-center justify-between">
							<div className="flex items-center gap-3">
								<div className="rounded-lg bg-default p-2 text-warning">
									<IconBolt size={20} />
								</div>
								<div>
									<h3 className="text-lg font-bold">Slash Lighting</h3>
									<p className="text-xs text-default-500">
										Back lid lighting control.
									</p>
								</div>
							</div>
							<Switch
								isSelected={slashEnabled}
								onChange={setSlashEnabled}
								isDisabled={!slashControlEnabled || !!busyAction}
								size="sm"
								aria-label="Enable slash lighting"
							>
								<Switch.Control>
									<Switch.Thumb />
								</Switch.Control>
							</Switch>
						</div>

						<Separator />

						<div className="space-y-6">
							{!slashControlEnabled ? (
								<p className="text-sm text-default-500">
									Slash controls are unavailable on this device or backend.
								</p>
							) : null}
							<div className="grid grid-cols-2 gap-4">
								<div className="space-y-1">
									<Label className="text-[10px] font-bold text-default-400 uppercase tracking-wider">
										Brightness
									</Label>
									<Input
										type="number"
										min={0}
										max={255}
										value={slashBrightness}
										onChange={(event) => setSlashBrightness(event.target.value)}
										disabled={!slashControlEnabled || !!busyAction}
									/>
								</div>
								<div className="space-y-1">
									<Label className="text-[10px] font-bold text-default-400 uppercase tracking-wider">
										Interval
									</Label>
									<Input
										type="number"
										min={0}
										max={255}
										value={slashInterval}
										onChange={(event) => setSlashInterval(event.target.value)}
										disabled={!slashControlEnabled || !!busyAction}
									/>
								</div>
							</div>

							<div className="grid grid-cols-1 gap-3 md:grid-cols-2">
								<Switch
									isSelected={slashShowOnBoot}
									onChange={setSlashShowOnBoot}
									isDisabled={!slashControlEnabled || !!busyAction}
								>
									<Switch.Control>
										<Switch.Thumb />
									</Switch.Control>
									<Switch.Content>
										<Label>Show on Boot</Label>
									</Switch.Content>
								</Switch>
								<Switch
									isSelected={slashShowOnSleep}
									onChange={setSlashShowOnSleep}
									isDisabled={!slashControlEnabled || !!busyAction}
								>
									<Switch.Control>
										<Switch.Thumb />
									</Switch.Control>
									<Switch.Content>
										<Label>Show on Sleep</Label>
									</Switch.Content>
								</Switch>
								<Switch
									isSelected={slashShowOnShutdown}
									onChange={setSlashShowOnShutdown}
									isDisabled={!slashControlEnabled || !!busyAction}
								>
									<Switch.Control>
										<Switch.Thumb />
									</Switch.Control>
									<Switch.Content>
										<Label>Show on Shutdown</Label>
									</Switch.Content>
								</Switch>
								<Switch
									isSelected={slashShowOnBattery}
									onChange={setSlashShowOnBattery}
									isDisabled={!slashControlEnabled || !!busyAction}
								>
									<Switch.Control>
										<Switch.Thumb />
									</Switch.Control>
									<Switch.Content>
										<Label>Show on Battery</Label>
									</Switch.Content>
								</Switch>
								<Switch
									isSelected={slashShowBatteryWarning}
									onChange={setSlashShowBatteryWarning}
									isDisabled={!slashControlEnabled || !!busyAction}
								>
									<Switch.Control>
										<Switch.Thumb />
									</Switch.Control>
									<Switch.Content>
										<Label>Show Battery Warning</Label>
									</Switch.Content>
								</Switch>
								<Switch
									isSelected={slashShowOnLidClosed}
									onChange={setSlashShowOnLidClosed}
									isDisabled={!slashControlEnabled || !!busyAction}
								>
									<Switch.Control>
										<Switch.Thumb />
									</Switch.Control>
									<Switch.Content>
										<Label>Show on Lid Closed</Label>
									</Switch.Content>
								</Switch>
							</div>

							<Select
								selectedKey={slashMode}
								onSelectionChange={(key) => setSlashMode(String(key))}
								isDisabled={!slashControlEnabled || !!busyAction}
							>
								<Label>Slash Mode</Label>
								<Select.Trigger>
									<Select.Value />
									<Select.Indicator />
								</Select.Trigger>
								<Select.Popover>
									<ListBox
										aria-label="Slash mode options"
										selectedKeys={[slashMode]}
										onSelectionChange={(keys) => {
											const key = Array.from(keys)[0];
											if (key) setSlashMode(String(key));
										}}
									>
										{slashModes.map((mode) => (
											<ListBox.Item key={mode} id={mode} textValue={mode}>
												{mode}
												<ListBox.ItemIndicator />
											</ListBox.Item>
										))}
									</ListBox>
								</Select.Popover>
							</Select>
						</div>

						<div className="grid grid-cols-2 gap-3 mt-auto">
							<Button
								className="font-bold"
								isDisabled={!slashControlEnabled || !!busyAction}
								onPress={() =>
									runDashboardAction("setSlashMode", () =>
										commands.setSlashMode({ mode: slashMode }),
									)
								}
							>
								Apply Mode
							</Button>
							<Button
								className="font-bold"
								isDisabled={!slashControlEnabled || !!busyAction}
								onPress={async () => {
									const updatePlan = [
										{
											action: "setSlashEnabled",
											call: () =>
												commands.setSlashEnabled({ enabled: slashEnabled }),
										},
										{
											action: "setSlashBrightness",
											call: () =>
												commands.setSlashBrightness({
													brightness: clampByte(slashBrightness),
												}),
										},
										{
											action: "setSlashInterval",
											call: () =>
												commands.setSlashInterval({
													interval: clampByte(slashInterval),
												}),
										},
										{
											action: "setSlashShowOnBoot",
											call: () =>
												commands.setSlashShowOnBoot({
													enabled: slashShowOnBoot,
												}),
										},
										{
											action: "setSlashShowOnSleep",
											call: () =>
												commands.setSlashShowOnSleep({
													enabled: slashShowOnSleep,
												}),
										},
										{
											action: "setSlashShowOnShutdown",
											call: () =>
												commands.setSlashShowOnShutdown({
													enabled: slashShowOnShutdown,
												}),
										},
										{
											action: "setSlashShowOnBattery",
											call: () =>
												commands.setSlashShowOnBattery({
													enabled: slashShowOnBattery,
												}),
										},
										{
											action: "setSlashShowBatteryWarning",
											call: () =>
												commands.setSlashShowBatteryWarning({
													enabled: slashShowBatteryWarning,
												}),
										},
										{
											action: "setSlashShowOnLidClosed",
											call: () =>
												commands.setSlashShowOnLidClosed({
													enabled: slashShowOnLidClosed,
												}),
										},
									] as const;
									for (const item of updatePlan) {
										const result = await runDashboardAction(
											item.action,
											item.call,
										);
										if (!result) {
											break;
										}
									}
								}}
							>
								Sync Slash
							</Button>
						</div>
					</div>
				</Card>
			</div>
		</div>
	);
}
