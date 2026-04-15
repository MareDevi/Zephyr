import {
	Button,
	Card,
	Label,
	ListBox,
	NumberField,
	Select,
	Separator,
	Switch,
} from "@heroui/react";
import { IconBolt, IconPalette } from "@tabler/icons-react";
import { useEffect, useMemo } from "react";
import { commands } from "../bindings";
import {
	extractOptions,
	useDashboardRuntime,
} from "../features/dashboard/runtime";
import { useDraftState } from "../features/dashboard/useDraftState";

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

function clampByte(value: number): number {
	return Math.max(0, Math.min(255, Math.round(value)));
}

export function LightingPage() {
	const { snapshot, busyAction, runDashboardAction } = useDashboardRuntime();

	const auraBrightness = useDraftState(2);
	const auraMode = useDraftState("Static");
	const slashEnabled = useDraftState(true);
	const slashBrightness = useDraftState(128);
	const slashInterval = useDraftState(0);
	const slashMode = useDraftState("Static");
	const slashShowOnBoot = useDraftState(false);
	const slashShowOnSleep = useDraftState(false);
	const slashShowOnShutdown = useDraftState(false);
	const slashShowOnBattery = useDraftState(false);
	const slashShowBatteryWarning = useDraftState(false);
	const slashShowOnLidClosed = useDraftState(false);

	useEffect(() => {
		if (!snapshot) {
			return;
		}
		if (snapshot.aura.brightness) {
			const match = snapshot.aura.brightness.match(/\d+/);
			if (match) {
				auraBrightness.syncFromSnapshot(Number.parseInt(match[0], 10) || 0);
			}
		}
		if (snapshot.aura.ledMode) {
			auraMode.syncFromSnapshot(snapshot.aura.ledMode.replace(/"/g, ""));
		}
		if (snapshot.slash.enabled != null) {
			slashEnabled.syncFromSnapshot(snapshot.slash.enabled);
		}
		if (snapshot.slash.brightness != null) {
			slashBrightness.syncFromSnapshot(snapshot.slash.brightness);
		}
		if (snapshot.slash.interval != null) {
			slashInterval.syncFromSnapshot(snapshot.slash.interval);
		}
		if (snapshot.slash.mode) {
			slashMode.syncFromSnapshot(snapshot.slash.mode.replace(/"/g, ""));
		}
		if (snapshot.slash.showOnBoot != null) {
			slashShowOnBoot.syncFromSnapshot(snapshot.slash.showOnBoot);
		}
		if (snapshot.slash.showOnSleep != null) {
			slashShowOnSleep.syncFromSnapshot(snapshot.slash.showOnSleep);
		}
		if (snapshot.slash.showOnShutdown != null) {
			slashShowOnShutdown.syncFromSnapshot(snapshot.slash.showOnShutdown);
		}
		if (snapshot.slash.showOnBattery != null) {
			slashShowOnBattery.syncFromSnapshot(snapshot.slash.showOnBattery);
		}
		if (snapshot.slash.showBatteryWarning != null) {
			slashShowBatteryWarning.syncFromSnapshot(snapshot.slash.showBatteryWarning);
		}
		if (snapshot.slash.showOnLidClosed != null) {
			slashShowOnLidClosed.syncFromSnapshot(snapshot.slash.showOnLidClosed);
		}
	}, [
		snapshot,
		auraBrightness,
		auraMode,
		slashEnabled,
		slashBrightness,
		slashInterval,
		slashMode,
		slashShowOnBoot,
		slashShowOnSleep,
		slashShowOnShutdown,
		slashShowOnBattery,
		slashShowBatteryWarning,
		slashShowOnLidClosed,
	]);

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
							<NumberField
								minValue={0}
								maxValue={3}
								step={1}
								value={auraBrightness.value}
								onChange={(value) => {
									if (Number.isFinite(value)) {
										auraBrightness.setFromUser(Math.round(value));
									}
								}}
								isDisabled={!auraControlEnabled || !!busyAction}
							>
								<Label>Brightness</Label>
								<NumberField.Group>
									<NumberField.DecrementButton />
									<NumberField.Input />
									<NumberField.IncrementButton />
								</NumberField.Group>
							</NumberField>

							<Select
								selectedKey={auraMode.value}
								onSelectionChange={(key) => auraMode.setFromUser(String(key))}
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
										selectedKeys={[auraMode.value]}
										onSelectionChange={(keys) => {
											const key = Array.from(keys)[0];
											if (key) auraMode.setFromUser(String(key));
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
									void (async () => {
										const result = await runDashboardAction("setAuraBrightness", () =>
										commands.setAuraBrightness({
											level: auraBrightness.value,
										}),
										);
										if (result) {
											auraBrightness.markClean();
										}
									})()
								}
							>
								Apply Brightness
							</Button>
							<Button
								className="font-bold"
								isDisabled={!auraControlEnabled || !!busyAction}
								onPress={() =>
									void (async () => {
										const result = await runDashboardAction("setAuraMode", () =>
											commands.setAuraMode({ mode: auraMode.value }),
										);
										if (result) {
											auraMode.markClean();
										}
									})()
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
								isSelected={slashEnabled.value}
								onChange={slashEnabled.setFromUser}
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
									<NumberField
										minValue={0}
										maxValue={255}
										value={slashBrightness.value}
										onChange={(value) => {
											if (Number.isFinite(value)) {
												slashBrightness.setFromUser(Math.round(value));
											}
										}}
										isDisabled={!slashControlEnabled || !!busyAction}
									>
										<Label className="text-[10px] font-bold text-default-400 uppercase tracking-wider">
											Brightness
										</Label>
										<NumberField.Group>
											<NumberField.DecrementButton />
											<NumberField.Input />
											<NumberField.IncrementButton />
										</NumberField.Group>
									</NumberField>
								</div>
								<div className="space-y-1">
									<NumberField
										minValue={0}
										maxValue={255}
										value={slashInterval.value}
										onChange={(value) => {
											if (Number.isFinite(value)) {
												slashInterval.setFromUser(Math.round(value));
											}
										}}
										isDisabled={!slashControlEnabled || !!busyAction}
									>
										<Label className="text-[10px] font-bold text-default-400 uppercase tracking-wider">
											Interval
										</Label>
										<NumberField.Group>
											<NumberField.DecrementButton />
											<NumberField.Input />
											<NumberField.IncrementButton />
										</NumberField.Group>
									</NumberField>
								</div>
							</div>

							<div className="grid grid-cols-1 gap-3 md:grid-cols-2">
								<Switch
									isSelected={slashShowOnBoot.value}
									onChange={slashShowOnBoot.setFromUser}
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
									isSelected={slashShowOnSleep.value}
									onChange={slashShowOnSleep.setFromUser}
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
									isSelected={slashShowOnShutdown.value}
									onChange={slashShowOnShutdown.setFromUser}
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
									isSelected={slashShowOnBattery.value}
									onChange={slashShowOnBattery.setFromUser}
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
									isSelected={slashShowBatteryWarning.value}
									onChange={slashShowBatteryWarning.setFromUser}
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
									isSelected={slashShowOnLidClosed.value}
									onChange={slashShowOnLidClosed.setFromUser}
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
								selectedKey={slashMode.value}
								onSelectionChange={(key) => slashMode.setFromUser(String(key))}
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
										selectedKeys={[slashMode.value]}
										onSelectionChange={(keys) => {
											const key = Array.from(keys)[0];
											if (key) slashMode.setFromUser(String(key));
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
									void (async () => {
										const result = await runDashboardAction("setSlashMode", () =>
											commands.setSlashMode({ mode: slashMode.value }),
										);
										if (result) {
											slashMode.markClean();
										}
									})()
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
												commands.setSlashEnabled({
													enabled: slashEnabled.value,
												}),
										},
										{
											action: "setSlashBrightness",
											call: () =>
												commands.setSlashBrightness({
													brightness: clampByte(slashBrightness.value),
												}),
										},
										{
											action: "setSlashInterval",
											call: () =>
												commands.setSlashInterval({
													interval: clampByte(slashInterval.value),
												}),
										},
										{
											action: "setSlashShowOnBoot",
											call: () =>
												commands.setSlashShowOnBoot({
													enabled: slashShowOnBoot.value,
												}),
										},
										{
											action: "setSlashShowOnSleep",
											call: () =>
												commands.setSlashShowOnSleep({
													enabled: slashShowOnSleep.value,
												}),
										},
										{
											action: "setSlashShowOnShutdown",
											call: () =>
												commands.setSlashShowOnShutdown({
													enabled: slashShowOnShutdown.value,
												}),
										},
										{
											action: "setSlashShowOnBattery",
											call: () =>
												commands.setSlashShowOnBattery({
													enabled: slashShowOnBattery.value,
												}),
										},
										{
											action: "setSlashShowBatteryWarning",
											call: () =>
												commands.setSlashShowBatteryWarning({
													enabled: slashShowBatteryWarning.value,
												}),
										},
										{
											action: "setSlashShowOnLidClosed",
											call: () =>
												commands.setSlashShowOnLidClosed({
													enabled: slashShowOnLidClosed.value,
												}),
										},
									] as const;
									let allSucceeded = true;
									for (const item of updatePlan) {
										const result = await runDashboardAction(
											item.action,
											item.call,
										);
										if (!result) {
											allSucceeded = false;
											break;
										}
									}
									if (allSucceeded) {
										slashEnabled.markClean();
										slashBrightness.markClean();
										slashInterval.markClean();
										slashShowOnBoot.markClean();
										slashShowOnSleep.markClean();
										slashShowOnShutdown.markClean();
										slashShowOnBattery.markClean();
										slashShowBatteryWarning.markClean();
										slashShowOnLidClosed.markClean();
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
