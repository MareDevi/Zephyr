import {
	Button,
	Card,
	Checkbox,
	Label,
	ListBox,
	NumberField,
	Select,
	Tabs,
} from "@heroui/react";
import { useEffect, useMemo, useState } from "react";
import { commands } from "../bindings";
import {
	extractOptions,
	useDashboardRuntime,
} from "../features/dashboard/runtime";
import { useDraftState } from "../features/dashboard/useDraftState";

const PLATFORM_PROFILE_FALLBACK = [
	"balanced",
	"performance",
	"quiet",
	"low-power",
	"custom",
];

function PlatformProfileTab() {
	const { snapshot, busyAction, runDashboardAction } = useDashboardRuntime();
	const selectedProfile = useDraftState("balanced");
	const [setOnAc, setSetOnAc] = useState(false);
	const [setOnBattery, setSetOnBattery] = useState(false);

	const profileOptions = useMemo(
		() =>
			extractOptions(
				snapshot?.platform.platformProfileChoices,
				PLATFORM_PROFILE_FALLBACK,
			),
		[snapshot?.platform.platformProfileChoices],
	);

	useEffect(() => {
		if (snapshot?.platform.platformProfile) {
			selectedProfile.syncFromSnapshot(snapshot.platform.platformProfile);
		}
	}, [snapshot?.platform.platformProfile, selectedProfile]);

	const platformControlEnabled =
		snapshot?.interfaces.asusdPlatformAvailable ?? false;
	const supportsAcProfileTarget =
		snapshot?.platform.platformProfileOnAc != null;
	const supportsBatteryProfileTarget =
		snapshot?.platform.platformProfileOnBattery != null;

	return (
		<div className="flex flex-col gap-6">
			<Card className="dashboard-card p-6">
				<div className="flex flex-col gap-6">
					<div>
						<h3 className="text-lg font-bold">Platform Profile</h3>
						<p className="text-xs text-default-500 mt-1">
							Configure performance profiles provided by the ASUS daemon.
						</p>
					</div>

					<Select
						aria-label="Select Platform Profile"
						selectedKey={selectedProfile.value}
						onSelectionChange={(key) =>
							selectedProfile.setFromUser(String(key))
						}
						isDisabled={!platformControlEnabled || !!busyAction}
					>
						<Select.Trigger>
							<Select.Value />
							<Select.Indicator />
						</Select.Trigger>
						<Select.Popover>
							<ListBox
								aria-label="Platform profile options"
								selectedKeys={[selectedProfile.value]}
								onSelectionChange={(keys) => {
									const key = Array.from(keys)[0];
									if (key) selectedProfile.setFromUser(String(key));
								}}
							>
								{profileOptions.map((profile) => (
									<ListBox.Item key={profile} id={profile} textValue={profile}>
										{profile}
										<ListBox.ItemIndicator />
									</ListBox.Item>
								))}
							</ListBox>
						</Select.Popover>
					</Select>

					<div className="flex flex-col gap-3">
						<Checkbox
							isSelected={setOnAc}
							onChange={setSetOnAc}
							isDisabled={
								!platformControlEnabled ||
								!!busyAction ||
								!supportsAcProfileTarget
							}
						>
							<Checkbox.Control />
							<Checkbox.Content>
								<Label className="text-sm">Apply to AC profile</Label>
							</Checkbox.Content>
						</Checkbox>
						<Checkbox
							isSelected={setOnBattery}
							onChange={setSetOnBattery}
							isDisabled={
								!platformControlEnabled ||
								!!busyAction ||
								!supportsBatteryProfileTarget
							}
						>
							<Checkbox.Control />
							<Checkbox.Content>
								<Label className="text-sm">Apply to battery profile</Label>
							</Checkbox.Content>
						</Checkbox>
					</div>

					<Button
						className="font-bold"
						isDisabled={!platformControlEnabled || !!busyAction}
						onPress={() =>
							void (async () => {
								const result = await runDashboardAction("setPlatformProfile", () =>
								commands.setPlatformProfile({
									profile: selectedProfile.value,
									ac: supportsAcProfileTarget ? setOnAc : undefined,
									battery: supportsBatteryProfileTarget
										? setOnBattery
										: undefined,
								}),
								);
								if (result) {
									selectedProfile.markClean();
								}
							})()
						}
					>
						Set Profile
					</Button>
				</div>
			</Card>
		</div>
	);
}

function GpuProfileTab() {
	const { snapshot, busyAction, runDashboardAction } = useDashboardRuntime();
	const selectedGpuMode = useDraftState("Hybrid");

	const gpuModes = useMemo(() => {
		const parsed = extractOptions(snapshot?.gpu.supportedModes, []);
		if (parsed.length > 0) {
			return parsed;
		}
		if (snapshot?.gpu.mode) {
			return [snapshot.gpu.mode];
		}
		return [];
	}, [snapshot?.gpu.supportedModes, snapshot?.gpu.mode]);

	useEffect(() => {
		if (snapshot?.gpu.mode) {
			selectedGpuMode.syncFromSnapshot(snapshot.gpu.mode);
		}
	}, [snapshot?.gpu.mode, selectedGpuMode]);

	const gpuControlEnabled =
		snapshot?.interfaces.supergfxdInterfaceAvailable ?? false;

	return (
		<div className="flex flex-col gap-6">
			<Card className="dashboard-card p-6">
				<div className="flex flex-col gap-6">
					<h3 className="text-lg font-bold">GPU Profile</h3>
					<p className="text-xs text-default-500">
						Select a supergfxd mode for GPU routing and power behavior.
					</p>

					<Select
						aria-label="Select GPU Mode"
						selectedKey={selectedGpuMode.value}
						onSelectionChange={(key) =>
							selectedGpuMode.setFromUser(String(key))
						}
						isDisabled={!gpuControlEnabled || !gpuModes.length || !!busyAction}
					>
						<Select.Trigger>
							<Select.Value />
							<Select.Indicator />
						</Select.Trigger>
						<Select.Popover>
							<ListBox
								aria-label="GPU mode options"
								selectedKeys={[selectedGpuMode.value]}
								onSelectionChange={(keys) => {
									const key = Array.from(keys)[0];
									if (key) selectedGpuMode.setFromUser(String(key));
								}}
							>
								{gpuModes.map((mode) => (
									<ListBox.Item key={mode} id={mode} textValue={mode}>
										{mode}
										<ListBox.ItemIndicator />
									</ListBox.Item>
								))}
							</ListBox>
						</Select.Popover>
					</Select>

					{!gpuControlEnabled ? (
						<p className="text-xs text-warning">
							GPU controls are unavailable on this system.
						</p>
					) : null}

					<Button
						className="font-bold"
						isDisabled={!gpuControlEnabled || !gpuModes.length || !!busyAction}
						onPress={() =>
							void (async () => {
								const result = await runDashboardAction("setGpuMode", () =>
									commands.setGpuMode({ mode: selectedGpuMode.value }),
								);
								if (result) {
									selectedGpuMode.markClean();
								}
							})()
						}
					>
						Apply GPU Mode
					</Button>
				</div>
			</Card>
		</div>
	);
}

function PowerProfileTab() {
	const { snapshot, busyAction, runDashboardAction } = useDashboardRuntime();
	const chargeLimit = useDraftState(80);

	useEffect(() => {
		if (snapshot?.platform.chargeControlEndThreshold != null) {
			chargeLimit.syncFromSnapshot(snapshot.platform.chargeControlEndThreshold);
		}
	}, [snapshot?.platform.chargeControlEndThreshold, chargeLimit]);

	const chargeControlEnabled =
		(snapshot?.interfaces.asusdPlatformAvailable ?? false) &&
		snapshot?.platform.chargeControlEndThreshold != null;

	return (
		<div className="flex flex-col gap-6">
			<Card className="dashboard-card p-6">
				<div className="flex flex-col gap-6">
					<div>
						<h3 className="text-lg font-bold">Charge Limit</h3>
						<p className="text-xs text-default-500 mt-1">
							Protect your battery health by limiting the maximum charge level.
						</p>
					</div>

					<div className="space-y-8">
						<NumberField
							className="w-full max-w-md"
							aria-label="Charge Limit"
							minValue={20}
							maxValue={100}
							step={1}
							value={chargeLimit.value}
							onChange={(value) => {
								if (Number.isFinite(value)) {
									chargeLimit.setFromUser(Math.round(value));
								}
							}}
							isDisabled={!chargeControlEnabled || !!busyAction}
						>
							<Label>Charge Limit</Label>
							<NumberField.Group>
								<NumberField.DecrementButton />
								<NumberField.Input className="text-xl font-bold" />
								<NumberField.IncrementButton />
							</NumberField.Group>
						</NumberField>

						<div className="rounded-xl bg-default-100 p-4 text-xs text-default-500">
							<p className="font-semibold text-foreground mb-1">
								Battery Health Tip
							</p>
							Setting a limit below 80% can significantly extend the long-term
							lifespan of your laptop battery.
						</div>
					</div>

					<Button
						className="font-bold"
						isDisabled={!chargeControlEnabled || !!busyAction}
						onPress={() =>
							void (async () => {
								const result = await runDashboardAction("setChargeLimit", () =>
									commands.setChargeLimit({
										percent: chargeLimit.value,
									}),
								);
								if (result) {
									chargeLimit.markClean();
								}
							})()
						}
					>
						Update Charge Policy
					</Button>
				</div>
			</Card>
		</div>
	);
}

export function ProfilesPage() {
	return (
		<div className="space-y-6">
			<Card className="dashboard-card p-4">
				<p className="section-title">System profiles</p>
				<p className="section-description mt-1">
					Configure performance strategy, GPU routing, and charging policy.
				</p>
			</Card>
			<Tabs className="w-full" aria-label="Profile tabs">
				<Tabs.ListContainer className="w-full">
					<Tabs.List
						aria-label="Profile tabs"
						className="w-full *:data-[selected=true]:text-accent-foreground"
					>
						<Tabs.Tab id="platform">
							Platform
							<Tabs.Indicator className="bg-accent" />
						</Tabs.Tab>
						<Tabs.Tab id="gpu">
							GPU
							<Tabs.Indicator className="bg-accent" />
						</Tabs.Tab>
						<Tabs.Tab id="power">
							Power
							<Tabs.Indicator className="bg-accent" />
						</Tabs.Tab>
					</Tabs.List>
				</Tabs.ListContainer>
				<Tabs.Panel className="pt-4" id="platform">
					<PlatformProfileTab />
				</Tabs.Panel>
				<Tabs.Panel className="pt-4" id="gpu">
					<GpuProfileTab />
				</Tabs.Panel>
				<Tabs.Panel className="pt-4" id="power">
					<PowerProfileTab />
				</Tabs.Panel>
			</Tabs>
		</div>
	);
}
